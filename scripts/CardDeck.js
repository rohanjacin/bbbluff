const hre = require('hardhat');

// Interface to deploy the deck of cards
class CardDeck {

	constructor () {
		this.carddeck = null;
		this.signer = null;
	}
}

// Deploys the CardDeck contract
CardDeck.prototype.deploy = async function () {

	let _carddeck = await hre.ethers.getContractFactory('CardDeck');
	this.carddeck = await _carddeck.deploy();
	await this.carddeck.waitForDeployment();
	const _address = await this.carddeck.getAddress();
	console.log(`Deployed to ${_address}`);

	let _signers = await hre.ethers.getSigners();
	this.signer = _signers[0];

	console.log("signer:", this.signer.address);

	this.mint();
}

// Mints a single card
CardDeck.prototype.mint = async function () {

	let _id = 1;
	let _proof = "0x21373022272527af1283e01282b202d";

	console.log("Minting a card..");
	await this.carddeck.connect(this.signer).mint(_id, _proof);

	console.log("Minted a card");

	_id = 2;
	_proof = "0x21373022272527af3434847ef2d";

	console.log("Minting a card..");
	await this.carddeck.connect(this.signer).mint(_id, _proof);

	console.log("Minted a card");

/*	_id = 1;
	let _proof2 = await this.carddeck.connect(this.signer).getMint(_id);

	console.log("_proof2:", _proof2);*/
}

var carddecknet = new CardDeck();

carddecknet.deploy().catch((error) => {
	console.error(error);
	process.exitCode = 1;
})