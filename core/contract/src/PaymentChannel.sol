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

contract PaymentChannel {
    bool isInit;

    uint256 public channelId; // TODO: Could be like a hash of the sender infos or just a counter identifier

    address public sender;
    address public recipient;
    uint256 public expiration; // Timeout in case the recipient never closes.

    uint256 public balance; // Token balance
    IERC20 public token; // Token address

    uint256 public price; // Price per API request decided by the recipient

    event channelCreated(
        uint channel_id,
        address sender,
        address recipient,
        uint256 expiration,
        uint256 balance,
        uint price,
        uint nonce
    );

    event channelClosed(
        uint channel_id,
        address sender,
        address recipient,
        uint256 timestamp,
        uint256 amount,
        uint256 nonce
    );

    event depositMade(
        uint channel_id,
        address sender,
        address recipient,
        uint256 amount,
        uint256 newBalance
    );

    event expirationExtended(
        uint channel_id,
        address sender,
        address recipient,
        uint256 expiration
    );

    event timeoutClaimed(
        uint channel_id,
        address sender,
        address recipient,
        uint256 timestamp
    );

    // Initialize the channel
    // NOTE: Needs the sender to approve the contract to spend the token amount
    function init(
        address _recipient,
        address _sender,
        uint256 _duration,
        address _tokenAddress,
        uint256 _amount,
        uint256 _price,
        uint256 _channelId
    ) public {
        require(!isInit, "Channel already initialized");
        sender = _sender;
        recipient = _recipient;
        expiration = block.timestamp + _duration;

        token = IERC20(_tokenAddress);
        // Transfer the token amount to the contract from the calling contracts
        token.transferFrom(msg.sender, address(this), _amount);
        balance = _amount;

        price = _price;
        channelId = _channelId;

        isInit = true;

        emit channelCreated(
            channelId,
            sender,
            recipient,
            expiration,
            balance,
            price,
            0
        );
    }

    // Deposit new token amount to the contract
    function deposit(uint _amount) external {
        require(msg.sender == sender, "Only sender can deposit");
        token.transferFrom(sender, address(this), _amount);
        balance += _amount;

        emit depositMade(channelId, sender, recipient, _amount, balance);
    }

    // Closing the channel using the senders signature to claim the amount & transfer the amount to the recipient
    // NOTE: Edge case possible, that does the contract owner
    function close(
        uint256 totalAmount, // the amount of credits used
        uint256 nonce,
        bytes calldata rawBody,
        bytes calldata signature
    ) public {
        require(msg.sender == recipient);

        // Verify the signature
        bytes32 messageHash = keccak256(
            abi.encode(channelId, totalAmount, nonce, rawBody)
        );
        bytes32 ethSignedMessageHash = getEthSignedMessageHash(messageHash);

        require(
            recoverSigner(ethSignedMessageHash, signature) == sender,
            "Invalid Signature"
        );

        // Token transfer to the recipient
        token.transfer(recipient, totalAmount);

        // Transfer the remaining balance to the sender
        uint256 remainingBalance = token.balanceOf(address(this));
        if (remainingBalance > 0) {
            token.transfer(sender, remainingBalance);
        }

        emit channelClosed(
            channelId,
            sender,
            recipient,
            block.timestamp,
            totalAmount,
            nonce
        );
    }

    // Extend the expiration time for the contract
    function extend(uint256 newExpiration) public {
        require(msg.sender == sender);
        require(newExpiration > expiration);
        expiration = newExpiration;

        emit expirationExtended(channelId, sender, recipient, expiration);
    }

    // Claim the remaining balance after the expiration time
    function claimTimeout() public {
        require(block.timestamp >= expiration);
        token.transfer(recipient, token.balanceOf(address(this)));

        emit timeoutClaimed(channelId, sender, recipient, block.timestamp);
    }

    function getBalance() public view returns (uint256) {
        return token.balanceOf(address(this));
    }

    //######  UTILITY FUNCTIONS ######//

    function recoverSigner(
        bytes32 _ethSignedMessageHash,
        bytes memory _signature
    ) public pure returns (address) {
        (bytes32 r, bytes32 s, uint8 v) = splitSignature(_signature);

        return ecrecover(_ethSignedMessageHash, v, r, s);
    }

    function splitSignature(
        bytes memory sig
    ) public pure returns (bytes32 r, bytes32 s, uint8 v) {
        require(sig.length == 65, "invalid signature length");

        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
    }

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
