# Bandot

introduce
===

Bandot stable coin. http://bandot.io/

Bandot is a decentralized and smart contract management stable coin that holders can use to collateralize their flowable digital assets.

Users between parachains can convert to a proprietary smart token by bailing the corresponding number of BDTs to the Bandot repeater. Using the interconnection of Bandot network service gateways to form a smart token circulation network, the token economy between different parachains can be exchanged for instant and safer and more convenient.

From the perspective of this article, a trustworthy stabilization mechanism may not only reduce the risk of our existing major redemption methods, but also have a great chance to function as a value storage tool.

The Bandot protocol supports value interoperability between different parachains, enabling the trading on the Polkadot chain to be quickly traded and circulated. Users can issue smart tokens through the Bandot protocol and run them on parachains, automatically redeeming through the value anchoring of the Bandot algorithm and the pass-through on the parachains of the Polkadot Ecology. The prosperity of the general economy and the ease of circulation can reduce transaction costs. It will be better able to serve dApp development to enable developers to better design and distribute their own decentralized applications.

## Bandot Docker

### Building the bandot node image

To build your own image from the source, you can run the following command:
```bash
./docker/build-node.sh
```
NOTE: Building the image takes a while. Count at least 30min on a good machine.

### Building the bandot ui image

To build your own image from the source, you can run the following command:
```bash
./docker/build-ui.sh
```

### Start bandot docker container

Run the following command
```
docker-compose -f docker/docker-compose.yml up -d
```
You can access the UI via http://localhost:3000

## Operations

first Init pdot and init bdt, open cdp if needed

### pdot is the abstract token staked 

>init set the admin for mint and burn.
transfer is used for users

### bdt is the stable coin.

>init set the admin for mint and burn.
transfer is used for users

### cdp

> skrPrice is the price of staked token in market, leave for oracle

#### lock & free
> lock for stake tokens, then free locked tokens

#### draw & wipe
> draw for release stable coins, and wipe for give back stable coins to burn.

etc.


