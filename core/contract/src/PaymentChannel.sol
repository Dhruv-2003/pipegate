// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

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

    function balanceOf(address account) external view returns (uint256);
}

/**
 * @title PaymentChannel
 * @notice A payment channel contract that enables micropayments between API consumers and providers
 * @dev This contract allows for off-chain signed transactions that can be settled on-chain
 * The channel supports deposits, signature-based closures, and timeout mechanisms
 */
contract PaymentChannel {
    /// @notice Channel states for better state management
    enum ChannelState {
        Uninitialized,
        Open,
        Closed
    }

    /// @notice Current state of the payment channel
    ChannelState public channelState;

    /// @notice Unique identifier for this payment channel
    uint256 public channelId;

    /// @notice Address of the API consumer (payer)
    address public sender;

    /// @notice Address of the API provider (payee)
    address public recipient;

    /// @notice Timestamp when the channel expires and can be force-closed by sender
    uint256 public expiration;

    /// @notice ERC20 token used for payments in this channel
    IERC20 public token;

    /// @notice Price per API request in token units (set by recipient)
    uint256 public price;

    /// @notice Last processed nonce to prevent replay attacks
    uint256 public lastProcessedNonce;

    /// @notice Address authorized to initialize this channel (factory contract)
    address public factory;

    /// @notice Emitted when a new payment channel is created
    event ChannelCreated(
        uint256 indexed channelId,
        address indexed sender,
        address indexed recipient,
        uint256 expiration,
        uint256 initialBalance,
        uint256 price
    );

    /// @notice Emitted when a payment channel is closed via signature
    event ChannelClosed(
        uint256 indexed channelId,
        address indexed sender,
        address indexed recipient,
        uint256 timestamp,
        uint256 amountPaid,
        uint256 amountRefunded,
        uint256 finalNonce
    );

    /// @notice Emitted when additional funds are deposited to the channel
    event DepositMade(
        uint256 indexed channelId,
        address indexed sender,
        uint256 amount,
        uint256 newBalance
    );

    /// @notice Emitted when channel expiration is extended
    event ExpirationExtended(
        uint256 indexed channelId,
        uint256 oldExpiration,
        uint256 newExpiration
    );

    /// @notice Emitted when sender claims funds after timeout
    event TimeoutClaimed(
        uint256 indexed channelId,
        address indexed sender,
        uint256 amount,
        uint256 timestamp
    );

    /// @notice Restricts function access to only the factory contract
    modifier onlyFactory() {
        require(msg.sender == factory, "PaymentChannel: Only factory can call");
        _;
    }

    /// @notice Restricts function access to only the channel sender
    modifier onlySender() {
        require(msg.sender == sender, "PaymentChannel: Only sender can call");
        _;
    }

    /// @notice Restricts function access to only the channel recipient
    modifier onlyRecipient() {
        require(
            msg.sender == recipient,
            "PaymentChannel: Only recipient can call"
        );
        _;
    }

    /// @notice Ensures the channel is in the expected state
    modifier onlyInState(ChannelState expectedState) {
        require(
            channelState == expectedState,
            "PaymentChannel: Invalid channel state"
        );
        _;
    }

    /// @notice Ensures the channel has not expired
    modifier notExpired() {
        require(
            block.timestamp < expiration,
            "PaymentChannel: Channel has expired"
        );
        _;
    }

    /// @notice Sets the factory address during contract creation
    constructor() {
        // Mark as closed to prevent initialization of the implementation
        channelState = ChannelState.Closed;
    }

    /**
     * @notice Initializes a new payment channel
     * @dev Can only be called once by the factory contract
     * @param _recipient The API provider who will receive payments
     * @param _sender The API consumer who will make payments
     * @param _duration Duration in seconds before the channel can be force-closed
     * @param _tokenAddress Address of the ERC20 token to be used for payments
     * @param _amount Initial deposit amount from the sender
     * @param _price Price per API request in token units
     * @param _channelId Unique identifier for this channel
     */
    function init(
        address _recipient,
        address _sender,
        uint256 _duration,
        address _tokenAddress,
        uint256 _amount,
        uint256 _price,
        uint256 _channelId
    ) external onlyInState(ChannelState.Uninitialized) {
        require(
            _recipient != address(0),
            "PaymentChannel: Invalid recipient address"
        );
        require(
            _sender != address(0),
            "PaymentChannel: Invalid sender address"
        );
        require(
            _tokenAddress != address(0),
            "PaymentChannel: Invalid token address"
        );
        require(
            _amount > 0,
            "PaymentChannel: Initial amount must be greater than 0"
        );
        require(_price > 0, "PaymentChannel: Price must be greater than 0");
        require(
            _duration > 0,
            "PaymentChannel: Duration must be greater than 0"
        );

        // Set factory for proxy instances
        factory = msg.sender;

        sender = _sender;
        recipient = _recipient;
        expiration = block.timestamp + _duration;
        token = IERC20(_tokenAddress);
        price = _price;
        channelId = _channelId;
        channelState = ChannelState.Open;

        // Transfer initial deposit from factory to this contract
        require(
            token.transferFrom(msg.sender, address(this), _amount),
            "PaymentChannel: Initial deposit transfer failed"
        );

        emit ChannelCreated(
            channelId,
            sender,
            recipient,
            expiration,
            _amount,
            price
        );
    }

    /**
     * @notice Allows the sender to deposit additional funds to the channel
     * @dev Can only be called by the channel sender while channel is open
     * @param _amount Amount of tokens to deposit
     */
    function deposit(
        uint256 _amount
    ) external onlySender onlyInState(ChannelState.Open) notExpired {
        require(
            _amount > 0,
            "PaymentChannel: Deposit amount must be greater than 0"
        );

        require(
            token.transferFrom(sender, address(this), _amount),
            "PaymentChannel: Deposit transfer failed"
        );

        emit DepositMade(channelId, sender, _amount, getBalance());
    }

    /**
     * @notice Closes the channel using a signature from the sender
     * @dev Can only be called by the recipient. Validates signature and prevents replay attacks
     * @param channelBalance The remaining balance that should stay in the channel
     * @param nonce Nonce for replay protection (must be greater than lastProcessedNonce)
     * @param rawBody Additional data that was signed (for extensibility)
     * @param signature Sender's signature authorizing the payment
     */
    function close(
        uint256 channelBalance,
        uint256 nonce,
        bytes calldata rawBody,
        bytes calldata signature
    ) external onlyRecipient onlyInState(ChannelState.Open) {
        require(nonce > lastProcessedNonce, "PaymentChannel: Invalid nonce");

        uint256 currentBalance = getBalance();
        require(
            channelBalance <= currentBalance,
            "PaymentChannel: Invalid channel balance"
        );

        // Verify the signature
        bytes32 messageHash = keccak256(
            abi.encodePacked(channelId, channelBalance, nonce, rawBody)
        );
        bytes32 ethSignedMessageHash = getEthSignedMessageHash(messageHash);

        require(
            recoverSigner(ethSignedMessageHash, signature) == sender,
            "PaymentChannel: Invalid signature"
        );

        // Calculate payment amount (difference between current balance and remaining balance)
        uint256 paymentAmount = currentBalance - channelBalance;

        // Update state before external calls
        channelState = ChannelState.Closed;
        lastProcessedNonce = nonce;

        // Transfer payment to recipient
        if (paymentAmount > 0) {
            require(
                token.transfer(recipient, paymentAmount),
                "PaymentChannel: Payment transfer failed"
            );
        }

        // Transfer remaining balance back to sender
        if (channelBalance > 0) {
            require(
                token.transfer(sender, channelBalance),
                "PaymentChannel: Refund transfer failed"
            );
        }

        emit ChannelClosed(
            channelId,
            sender,
            recipient,
            block.timestamp,
            paymentAmount,
            channelBalance,
            nonce
        );
    }

    /**
     * @notice Extends the expiration time of the channel
     * @dev Can only be called by the sender while channel is open
     * @param newExpiration New expiration timestamp (must be later than current expiration)
     */
    function extend(
        uint256 newExpiration
    ) external onlySender onlyInState(ChannelState.Open) {
        require(
            newExpiration > expiration,
            "PaymentChannel: New expiration must be later"
        );
        require(
            newExpiration > block.timestamp,
            "PaymentChannel: New expiration must be in the future"
        );

        uint256 oldExpiration = expiration;
        expiration = newExpiration;

        emit ExpirationExtended(channelId, oldExpiration, newExpiration);
    }

    /**
     * @notice Allows sender to claim all remaining funds after the channel expires
     * @dev Can only be called after expiration timestamp has passed
     */
    function claimTimeout() external onlySender onlyInState(ChannelState.Open) {
        require(
            block.timestamp >= expiration,
            "PaymentChannel: Channel has not expired yet"
        );

        uint256 balance = getBalance();
        require(balance > 0, "PaymentChannel: No funds to claim");

        // Update state before external call
        channelState = ChannelState.Closed;

        require(
            token.transfer(sender, balance),
            "PaymentChannel: Timeout claim transfer failed"
        );

        emit TimeoutClaimed(channelId, sender, balance, block.timestamp);
    }

    /**
     * @notice Returns the current token balance of the channel
     * @return The balance of tokens held by this contract
     */
    function getBalance() public view returns (uint256) {
        return token.balanceOf(address(this));
    }

    /**
     * @notice Returns comprehensive channel information
     * @return id Channel ID
     * @return senderAddr Sender address
     * @return recipientAddr Recipient address
     * @return exp Expiration timestamp
     * @return balance Current token balance
     * @return pricePerRequest Price per API request
     * @return lastNonce Last processed nonce
     * @return state Current channel state
     */
    function getChannelInfo()
        external
        view
        returns (
            uint256 id,
            address senderAddr,
            address recipientAddr,
            uint256 exp,
            uint256 balance,
            uint256 pricePerRequest,
            uint256 lastNonce,
            ChannelState state
        )
    {
        return (
            channelId,
            sender,
            recipient,
            expiration,
            getBalance(),
            price,
            lastProcessedNonce,
            channelState
        );
    }

    //######  SIGNATURE VERIFICATION UTILITIES ######//

    /**
     * @notice Recovers the signer address from an Ethereum signed message hash and signature
     * @param _ethSignedMessageHash The hash of the signed message
     * @param _signature The signature bytes
     * @return The address of the signer
     */
    function recoverSigner(
        bytes32 _ethSignedMessageHash,
        bytes memory _signature
    ) public pure returns (address) {
        (bytes32 r, bytes32 s, uint8 v) = splitSignature(_signature);
        return ecrecover(_ethSignedMessageHash, v, r, s);
    }

    /**
     * @notice Splits a signature into its r, s, v components
     * @param sig The signature bytes to split
     * @return r The r component of the signature
     * @return s The s component of the signature
     * @return v The v component of the signature
     */
    function splitSignature(
        bytes memory sig
    ) public pure returns (bytes32 r, bytes32 s, uint8 v) {
        require(sig.length == 65, "PaymentChannel: Invalid signature length");

        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
    }

    /**
     * @notice Converts a message hash to an Ethereum signed message hash
     * @param _messageHash The original message hash
     * @return The Ethereum signed message hash
     */
    function getEthSignedMessageHash(
        bytes32 _messageHash
    ) public pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    "\x19Ethereum Signed Message:\n32",
                    _messageHash
                )
            );
    }
}
