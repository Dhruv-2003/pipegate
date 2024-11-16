// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {PaymentChannel} from "./PaymentChannel.sol";
import {Proxy} from "./MinimalProxy.sol";

// API providers could register for once, listing their token pricing and other details.
// Channel creation will take place from here, and it will be assigned a channel Id
contract ChannelFactory {
    mapping(address => uint) public pricing;
    mapping(uint => address) public channels;

    uint public totalChannels;

    address paymentChannelImplementation;

    event pricingRegistered(
        address indexed recipient,
        uint price,
        uint timestamp
    );

    event channelCreated(
        uint indexed channelId,
        address indexed sender,
        address indexed recipient,
        uint256 duration,
        address tokenAddress,
        uint256 amount,
        uint256 price,
        uint256 timestamp
    );

    constructor() {
        paymentChannelImplementation = address(new PaymentChannel());
    }

    function register(uint price) public {
        address recipient = msg.sender;
        pricing[recipient] = price;
        emit pricingRegistered(recipient, price, block.timestamp);
    }

    function createChannel(
        address recipient,
        uint256 _duration,
        address _tokenAddress,
        uint256 _amount
    ) public {
        uint price = pricing[recipient];
        totalChannels++;

        address sender = msg.sender;

        address proxyAddress = address(new Proxy(paymentChannelImplementation));

        PaymentChannel channel = PaymentChannel(proxyAddress);

        channel.init(
            recipient,
            sender,
            _duration,
            _tokenAddress,
            _amount,
            price,
            totalChannels
        );

        channels[totalChannels] = address(channel);

        emit channelCreated(
            totalChannels,
            sender,
            recipient,
            _duration,
            _tokenAddress,
            _amount,
            price,
            block.timestamp
        );
    }
}
