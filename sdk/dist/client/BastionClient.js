"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BastionClient = void 0;
const web3_js_1 = require("@solana/web3.js");
const DarkPoolClient_1 = require("./DarkPoolClient");
class BastionClient {
    constructor(config) {
        this.connection = new web3_js_1.Connection(config.rpcUrl, config.commitment || 'confirmed');
        this.darkPool = new DarkPoolClient_1.DarkPoolClient({
            connection: this.connection,
            relayerUrl: config.relayerUrl,
            relayerPubKey: config.relayerPubKey
        });
    }
}
exports.BastionClient = BastionClient;
