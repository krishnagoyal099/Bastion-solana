"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseTransactionLogs = parseTransactionLogs;
const crypto = __importStar(require("crypto"));
function parseTransactionLogs(logs) {
    const events = [];
    for (const log of logs) {
        if (log.startsWith('Program data: ')) {
            try {
                const b64Data = log.replace('Program data: ', '').trim();
                const rawBytes = Buffer.from(b64Data, 'base64');
                const discriminator = rawBytes.slice(0, 8);
                const data = rawBytes.slice(8);
                const orderCommittedDisc = crypto.createHash('sha256').update('event:OrderCommitted').digest().slice(0, 8);
                const swapExecutedDisc = crypto.createHash('sha256').update('event:SwapExecuted').digest().slice(0, 8);
                if (discriminator.equals(orderCommittedDisc)) {
                    events.push({
                        name: 'OrderCommitted',
                        data: { rawHex: data.toString('hex') }
                    });
                }
                else if (discriminator.equals(swapExecutedDisc)) {
                    events.push({
                        name: 'SwapExecuted',
                        data: { rawHex: data.toString('hex') }
                    });
                }
            }
            catch (e) {
                console.error('Failed to parse log:', e);
            }
        }
    }
    return events;
}
