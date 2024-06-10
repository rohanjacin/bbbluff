//SPDX-License-Identifier: UNLICENSED

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";
import "hardhat/console.sol";

// Contract for a deck of cards with individual proofs
contract CardDeck is ERC1155 {

	// Number of cards in the deck
	uint8 public count;

	// An array sequence id vs card token id
	uint256 [] public cards;

	// Creator
	address public _admin;

	constructor() ERC1155 ("") {
		_admin = msg.sender;
		count = 0;
	}

	// Mint each card by supplying correct proof
	function mintCard (uint8 _seqid, uint256 _proofhash) public
		onlyAdmin onlyNotAllMinted onlyValidSeqId (_seqid) {

		cards[_seqid] = _proofhash;

		// mint the card ERC1155 token, token id is proofhash  
		_mint(msg.sender, _proofhash, 1, "");

		count++;
	}

	// Seal the card deck once shuffled
	function sealDeck (uint256 _deckproof) public
		onlyAdmin onlyAllMinted
		returns (bool success) {

		success = false;

		// Check for _deckproof
		success = true;
		return success;
	}

	// Get the cards i.e the card(s) proof's 
	function getCards () public view 
		onlyAllMinted
		returns (uint256 [] memory _cards) {

		// Query balance of admin/creator
		for (uint8 i = 0; i < count; i++) {
			_cards[i] = balanceOf(_admin, cards[i]);
		}

		return _cards;
	}

	modifier onlyAdmin () {
		if (_admin != msg.sender) revert();
		_;
	}

	modifier onlyNotAllMinted () {
		if (!(count < 52)) revert();
		_;
	}

	modifier onlyAllMinted () {
		if (count != 52) revert();
		_;
	}

	modifier onlyValidSeqId (uint8 _id) {
		if ((_id < 0) || (_id > 51)) revert();
		_;
	}

}
	

