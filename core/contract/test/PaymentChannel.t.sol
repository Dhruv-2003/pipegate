// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {PaymentChannel} from "../src/PaymentChannel.sol";
import {ChannelFactory} from "../src/ChannelFactory.sol";

// Mock ERC20 token for testing
contract MockERC20 {
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;
    
    string public name = "Mock Token";
    string public symbol = "MOCK";
    uint8 public decimals = 18;
    uint256 public totalSupply;
    
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    
    constructor(uint256 _totalSupply) {
        totalSupply = _totalSupply;
        balanceOf[msg.sender] = _totalSupply;
    }
    
    function transfer(address to, uint256 amount) external returns (bool) {
        require(balanceOf[msg.sender] >= amount, "Insufficient balance");
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        emit Transfer(msg.sender, to, amount);
        return true;
    }
    
    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        require(balanceOf[from] >= amount, "Insufficient balance");
        require(allowance[from][msg.sender] >= amount, "Insufficient allowance");
        
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        allowance[from][msg.sender] -= amount;
        
        emit Transfer(from, to, amount);
        return true;
    }
    
    function approve(address spender, uint256 amount) external returns (bool) {
        allowance[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }
    
    function mint(address to, uint256 amount) external {
        balanceOf[to] += amount;
        totalSupply += amount;
        emit Transfer(address(0), to, amount);
    }
}

contract PaymentChannelTest is Test {
    ChannelFactory public factory;
    PaymentChannel public paymentChannel;
    MockERC20 public token;
    
    address public sender = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8; // matches senderPrivateKey
    address public recipient = 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC; // matches recipientPrivateKey
    address public other = address(0x3);
    
    uint256 public constant INITIAL_BALANCE = 1000e18;
    uint256 public constant PRICE_PER_REQUEST = 1e18;
    uint256 public constant CHANNEL_DURATION = 30 days;
    uint256 public constant INITIAL_DEPOSIT = 100e18;
    
    uint256 private senderPrivateKey = 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d;
    uint256 private recipientPrivateKey = 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a;
    
    uint256 public channelId;
    address public channelAddress;
    
    event ChannelCreated(
        uint256 indexed channelId,
        address indexed sender,
        address indexed recipient,
        uint256 expiration,
        uint256 initialBalance,
        uint256 price
    );
    
    event ChannelClosed(
        uint256 indexed channelId,
        address indexed sender,
        address indexed recipient,
        uint256 timestamp,
        uint256 amountPaid,
        uint256 amountRefunded,
        uint256 finalNonce
    );
    
    event DepositMade(
        uint256 indexed channelId,
        address indexed sender,
        uint256 amount,
        uint256 newBalance
    );
    
    function setUp() public {
        // Deploy contracts
        token = new MockERC20(INITIAL_BALANCE * 10);
        factory = new ChannelFactory();
        
        // Set up accounts
        vm.deal(sender, 1 ether);
        vm.deal(recipient, 1 ether);
        vm.deal(other, 1 ether);
        
        // Distribute tokens
        token.transfer(sender, INITIAL_BALANCE);
        token.transfer(recipient, INITIAL_BALANCE);
        token.transfer(other, INITIAL_BALANCE);
        
        // Register recipient as API provider
        vm.prank(recipient);
        factory.register(PRICE_PER_REQUEST);
        
        // Create a payment channel for most tests
        vm.startPrank(sender);
        token.approve(address(factory), INITIAL_DEPOSIT);
        (channelId, channelAddress) = factory.createChannel(
            recipient,
            CHANNEL_DURATION,
            address(token),
            INITIAL_DEPOSIT
        );
        vm.stopPrank();
        
        paymentChannel = PaymentChannel(channelAddress);
    }
    
    function testChannelCreation() public {
        // Test channel was created with correct parameters
        assertEq(paymentChannel.sender(), sender);
        assertEq(paymentChannel.recipient(), recipient);
        assertEq(paymentChannel.price(), PRICE_PER_REQUEST);
        assertEq(paymentChannel.getBalance(), INITIAL_DEPOSIT);
        assertEq(uint256(paymentChannel.channelState()), uint256(PaymentChannel.ChannelState.Open));
        
        // Check expiration is approximately correct (within 1 minute tolerance)
        uint256 expectedExpiration = block.timestamp + CHANNEL_DURATION;
        uint256 actualExpiration = paymentChannel.expiration();
        assertApproxEqAbs(actualExpiration, expectedExpiration, 60);
    }
    
    function testDeposit() public {
        uint256 depositAmount = 50e18;
        uint256 initialBalance = paymentChannel.getBalance();
        
        vm.startPrank(sender);
        token.approve(address(paymentChannel), depositAmount);
        
        vm.expectEmit(true, true, false, true);
        emit DepositMade(paymentChannel.channelId(), sender, depositAmount, initialBalance + depositAmount);
        
        paymentChannel.deposit(depositAmount);
        vm.stopPrank();
        
        assertEq(paymentChannel.getBalance(), initialBalance + depositAmount);
    }
    
    function testDepositFailsForNonSender() public {
        uint256 depositAmount = 50e18;
        
        vm.startPrank(other);
        token.approve(address(paymentChannel), depositAmount);
        
        vm.expectRevert("PaymentChannel: Only sender can call");
        paymentChannel.deposit(depositAmount);
        vm.stopPrank();
    }
    
    function testDepositFailsForZeroAmount() public {
        vm.startPrank(sender);
        vm.expectRevert("PaymentChannel: Deposit amount must be greater than 0");
        paymentChannel.deposit(0);
        vm.stopPrank();
    }
    
    function testChannelClose() public {
        uint256 channelBalance = 80e18; // Leave 80 tokens in channel
        uint256 nonce = 1;
        bytes memory rawBody = "";
        
        // Create signature
        bytes32 messageHash = keccak256(
            abi.encodePacked(paymentChannel.channelId(), channelBalance, nonce, rawBody)
        );
        bytes32 ethSignedMessageHash = paymentChannel.getEthSignedMessageHash(messageHash);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(senderPrivateKey, ethSignedMessageHash);
        bytes memory signature = abi.encodePacked(r, s, v);
        
        uint256 initialRecipientBalance = token.balanceOf(recipient);
        uint256 initialSenderBalance = token.balanceOf(sender);
        uint256 paymentAmount = INITIAL_DEPOSIT - channelBalance;
        
        vm.expectEmit(true, true, true, true);
        emit ChannelClosed(
            paymentChannel.channelId(),
            sender,
            recipient,
            block.timestamp,
            paymentAmount,
            channelBalance,
            nonce
        );
        
        vm.prank(recipient);
        paymentChannel.close(channelBalance, nonce, rawBody, signature);
        
        // Check balances
        assertEq(token.balanceOf(recipient), initialRecipientBalance + paymentAmount);
        assertEq(token.balanceOf(sender), initialSenderBalance + channelBalance);
        assertEq(paymentChannel.getBalance(), 0);
        assertEq(uint256(paymentChannel.channelState()), uint256(PaymentChannel.ChannelState.Closed));
        assertEq(paymentChannel.lastProcessedNonce(), nonce);
    }
    
    function testChannelCloseFailsWithInvalidSignature() public {
        uint256 channelBalance = 80e18;
        uint256 nonce = 1;
        bytes memory rawBody = "";
        
        // Create signature with wrong private key
        bytes32 messageHash = keccak256(
            abi.encodePacked(paymentChannel.channelId(), channelBalance, nonce, rawBody)
        );
        bytes32 ethSignedMessageHash = paymentChannel.getEthSignedMessageHash(messageHash);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(recipientPrivateKey, ethSignedMessageHash);
        bytes memory signature = abi.encodePacked(r, s, v);
        
        vm.prank(recipient);
        vm.expectRevert("PaymentChannel: Invalid signature");
        paymentChannel.close(channelBalance, nonce, rawBody, signature);
    }
    
    function testChannelCloseFailsWithOldNonce() public {
        // The channel starts with lastProcessedNonce = 0
        // So nonce must be > 0. Let's try with nonce = 0 which should fail
        uint256 channelBalance = 90e18;
        uint256 nonce = 0; // This should fail since it's not > lastProcessedNonce (0)
        bytes memory rawBody = "";
        
        bytes32 messageHash = keccak256(
            abi.encodePacked(paymentChannel.channelId(), channelBalance, nonce, rawBody)
        );
        bytes32 ethSignedMessageHash = paymentChannel.getEthSignedMessageHash(messageHash);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(senderPrivateKey, ethSignedMessageHash);
        bytes memory signature = abi.encodePacked(r, s, v);
        
        vm.prank(recipient);
        vm.expectRevert("PaymentChannel: Invalid nonce");
        paymentChannel.close(channelBalance, nonce, rawBody, signature);
    }
    
    function testExtendExpiration() public {
        uint256 newExpiration = block.timestamp + CHANNEL_DURATION + 7 days;
        uint256 oldExpiration = paymentChannel.expiration();
        
        vm.prank(sender);
        paymentChannel.extend(newExpiration);
        
        assertEq(paymentChannel.expiration(), newExpiration);
        assert(paymentChannel.expiration() > oldExpiration);
    }
    
    function testExtendFailsForNonSender() public {
        uint256 newExpiration = block.timestamp + CHANNEL_DURATION + 7 days;
        
        vm.prank(other);
        vm.expectRevert("PaymentChannel: Only sender can call");
        paymentChannel.extend(newExpiration);
    }
    
    function testExtendFailsForPastExpiration() public {
        uint256 pastTime = block.timestamp > 1 days ? block.timestamp - 1 days : 0;
        
        vm.prank(sender);
        vm.expectRevert("PaymentChannel: New expiration must be later");
        paymentChannel.extend(pastTime);
    }
    
    function testClaimTimeout() public {
        // Fast forward past expiration
        vm.warp(block.timestamp + CHANNEL_DURATION + 1);
        
        uint256 initialSenderBalance = token.balanceOf(sender);
        uint256 channelBalance = paymentChannel.getBalance();
        
        vm.prank(sender);
        paymentChannel.claimTimeout();
        
        assertEq(token.balanceOf(sender), initialSenderBalance + channelBalance);
        assertEq(paymentChannel.getBalance(), 0);
        assertEq(uint256(paymentChannel.channelState()), uint256(PaymentChannel.ChannelState.Closed));
    }
    
    function testClaimTimeoutFailsBeforeExpiration() public {
        vm.prank(sender);
        vm.expectRevert("PaymentChannel: Channel has not expired yet");
        paymentChannel.claimTimeout();
    }
    
    function testClaimTimeoutFailsForNonSender() public {
        vm.warp(block.timestamp + CHANNEL_DURATION + 1);
        
        vm.prank(other);
        vm.expectRevert("PaymentChannel: Only sender can call");
        paymentChannel.claimTimeout();
    }
    
    function testGetChannelInfo() public {
        (
            uint256 id,
            address senderAddr,
            address recipientAddr,
            uint256 exp,
            uint256 balance,
            uint256 pricePerRequest,
            uint256 lastNonce,
            PaymentChannel.ChannelState state
        ) = paymentChannel.getChannelInfo();
        
        assertEq(id, paymentChannel.channelId());
        assertEq(senderAddr, sender);
        assertEq(recipientAddr, recipient);
        assertEq(exp, paymentChannel.expiration());
        assertEq(balance, INITIAL_DEPOSIT);
        assertEq(pricePerRequest, PRICE_PER_REQUEST);
        assertEq(lastNonce, 0);
        assertEq(uint256(state), uint256(PaymentChannel.ChannelState.Open));
    }
    
    function testSignatureUtilities() public {
        bytes32 messageHash = keccak256("test message");
        bytes32 ethSignedMessageHash = paymentChannel.getEthSignedMessageHash(messageHash);
        
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(senderPrivateKey, ethSignedMessageHash);
        bytes memory signature = abi.encodePacked(r, s, v);
        
        // Test signature splitting
        (bytes32 rRecovered, bytes32 sRecovered, uint8 vRecovered) = paymentChannel.splitSignature(signature);
        assertEq(rRecovered, r);
        assertEq(sRecovered, s);
        assertEq(vRecovered, v);
        
        // Test signer recovery
        address recoveredSigner = paymentChannel.recoverSigner(ethSignedMessageHash, signature);
        assertEq(recoveredSigner, sender);
    }
    
    function testSignatureUtilitiesFailsWithInvalidLength() public {
        bytes memory invalidSignature = new bytes(64); // Should be 65 bytes
        
        vm.expectRevert("PaymentChannel: Invalid signature length");
        paymentChannel.splitSignature(invalidSignature);
    }
}
