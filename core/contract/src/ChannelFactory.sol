// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {PaymentChannel} from "./PaymentChannel.sol";
import {Proxy} from "./MinimalProxy.sol";

interface IERC20 {
    function transferFrom(
        address sender,
        address recipient,
        uint256 amount
    ) external returns (bool);

    function transfer(
        address recipient,
        uint256 amount
    ) external returns (bool);

    function approve(address spender, uint256 amount) external returns (bool);

    function balanceOf(address account) external view returns (uint256);
}

/**
 * @title ChannelFactory
 * @notice Factory contract for creating and managing payment channels
 * @dev This contract allows API providers to register their pricing and enables
 * API consumers to create payment channels with minimal gas costs using proxy pattern
 */
contract ChannelFactory {
    /// @notice Mapping from API provider address to their price per request
    mapping(address => uint256) public pricing;
    
    /// @notice Mapping from channel ID to channel contract address
    mapping(uint256 => address) public channels;
    
    /// @notice Mapping from sender address to their channel IDs
    mapping(address => uint256[]) public senderChannels;
    
    /// @notice Mapping from recipient address to their channel IDs  
    mapping(address => uint256[]) public recipientChannels;
    
    /// @notice Total number of channels created
    uint256 public totalChannels;
    
    /// @notice Address of the PaymentChannel implementation contract
    address public immutable paymentChannelImplementation;
    
    /// @notice Minimum price that can be set by API providers
    uint256 public constant MINIMUM_PRICE = 1;
    
    /// @notice Maximum duration for a payment channel (365 days)
    uint256 public constant MAX_CHANNEL_DURATION = 365 days;

    /// @notice Emitted when an API provider registers their pricing
    event PricingRegistered(
        address indexed recipient,
        uint256 price,
        uint256 timestamp
    );

    /// @notice Emitted when a new payment channel is created
    event ChannelCreated(
        uint256 indexed channelId,
        address indexed channelAddress,
        address indexed sender,
        address recipient,
        uint256 duration,
        address tokenAddress,
        uint256 amount,
        uint256 price,
        uint256 timestamp
    );

    /// @notice Emitted when pricing is updated for an API provider
    event PricingUpdated(
        address indexed recipient,
        uint256 oldPrice,
        uint256 newPrice,
        uint256 timestamp
    );

    /**
     * @notice Constructor deploys the PaymentChannel implementation contract
     * @dev Uses immutable variable to save gas on channel creation
     */
    constructor() {
        paymentChannelImplementation = address(new PaymentChannel());
    }

    /**
     * @notice Registers an API provider with their price per request
     * @dev API providers must register before channels can be created for them
     * @param price Price per API request in token units (must be >= MINIMUM_PRICE)
     */
    function register(uint256 price) external {
        require(price >= MINIMUM_PRICE, "ChannelFactory: Price too low");
        
        address recipient = msg.sender;
        uint256 oldPrice = pricing[recipient];
        pricing[recipient] = price;
        
        if (oldPrice == 0) {
            emit PricingRegistered(recipient, price, block.timestamp);
        } else {
            emit PricingUpdated(recipient, oldPrice, price, block.timestamp);
        }
    }

    /**
     * @notice Creates a new payment channel between sender and recipient
     * @dev Uses minimal proxy pattern to reduce gas costs. Sender must approve tokens first
     * @param recipient Address of the API provider (must be registered)
     * @param _duration Duration of the channel in seconds (max MAX_CHANNEL_DURATION)
     * @param _tokenAddress Address of the ERC20 token for payments
     * @param _amount Initial deposit amount for the channel
     * @return channelId The ID of the newly created channel
     * @return channelAddress The address of the new payment channel contract
     */
    function createChannel(
        address recipient,
        uint256 _duration,
        address _tokenAddress,
        uint256 _amount
    ) external returns (uint256 channelId, address channelAddress) {
        require(recipient != address(0), "ChannelFactory: Invalid recipient address");
        require(_tokenAddress != address(0), "ChannelFactory: Invalid token address");
        require(_amount > 0, "ChannelFactory: Amount must be greater than 0");
        require(_duration > 0 && _duration <= MAX_CHANNEL_DURATION, "ChannelFactory: Invalid duration");
        
        uint256 price = pricing[recipient];
        require(price > 0, "ChannelFactory: Recipient not registered");
        
        address sender = msg.sender;
        require(sender != recipient, "ChannelFactory: Sender cannot be recipient");
        
        // Increment channel counter
        totalChannels++;
        channelId = totalChannels;

        // Transfer tokens from sender to factory first
        require(
            IERC20(_tokenAddress).transferFrom(sender, address(this), _amount),
            "ChannelFactory: Token transfer failed"
        );

        // Deploy new payment channel using minimal proxy
        channelAddress = address(new Proxy(paymentChannelImplementation));
        PaymentChannel channel = PaymentChannel(channelAddress);

        // Approve tokens for the channel contract
        require(
            IERC20(_tokenAddress).approve(channelAddress, _amount),
            "ChannelFactory: Token approval failed"
        );

        // Initialize the payment channel
        channel.init(
            recipient,
            sender,
            _duration,
            _tokenAddress,
            _amount,
            price,
            channelId
        );

        // Store channel mapping and user associations
        channels[channelId] = channelAddress;
        senderChannels[sender].push(channelId);
        recipientChannels[recipient].push(channelId);

        emit ChannelCreated(
            channelId,
            channelAddress,
            sender,
            recipient,
            _duration,
            _tokenAddress,
            _amount,
            price,
            block.timestamp
        );
    }

    /**
     * @notice Returns all channel IDs created by a specific sender
     * @param sender Address of the sender
     * @return Array of channel IDs
     */
    function getSenderChannels(address sender) external view returns (uint256[] memory) {
        return senderChannels[sender];
    }

    /**
     * @notice Returns all channel IDs for a specific recipient
     * @param recipient Address of the recipient
     * @return Array of channel IDs
     */
    function getRecipientChannels(address recipient) external view returns (uint256[] memory) {
        return recipientChannels[recipient];
    }

    /**
     * @notice Checks if an address is registered as an API provider
     * @param provider Address to check
     * @return True if the provider is registered with a price > 0
     */
    function isRegisteredProvider(address provider) external view returns (bool) {
        return pricing[provider] > 0;
    }

    /**
     * @notice Returns comprehensive information about a channel
     * @param channelId ID of the channel to query
     * @return exists Whether the channel exists
     * @return channelAddress Address of the channel contract
     * @return id Channel ID
     * @return sender Sender address
     * @return recipient Recipient address
     * @return expiration Expiration timestamp
     * @return balance Current balance
     * @return price Price per request
     * @return lastNonce Last processed nonce
     * @return state Channel state
     */
    function getChannelInfo(uint256 channelId) external view returns (
        bool exists,
        address channelAddress,
        uint256 id,
        address sender,
        address recipient,
        uint256 expiration,
        uint256 balance,
        uint256 price,
        uint256 lastNonce,
        PaymentChannel.ChannelState state
    ) {
        channelAddress = channels[channelId];
        exists = channelAddress != address(0);
        
        if (exists) {
            PaymentChannel channel = PaymentChannel(channelAddress);
            (id, sender, recipient, expiration, balance, price, lastNonce, state) = channel.getChannelInfo();
        }
    }
}
