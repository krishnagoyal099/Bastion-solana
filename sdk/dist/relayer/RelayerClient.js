"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.RelayerClient = void 0;
const axios_1 = __importDefault(require("axios"));
class RelayerClient {
    constructor(endpoint) {
        this.endpoint = endpoint;
    }
    async submitOrder(payload) {
        const response = await axios_1.default.post(`${this.endpoint}/order`, payload);
        return response.data;
    }
    async getHealth() {
        try {
            const response = await axios_1.default.get(`${this.endpoint}/health`);
            return response.data === 'OK';
        }
        catch {
            return false;
        }
    }
}
exports.RelayerClient = RelayerClient;
