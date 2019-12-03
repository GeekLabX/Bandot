import BN from 'bn.js';

export interface Cup {
    id: BN;
    locked_collaterals: BN;
    debts: BN;
    tax: BN;
}
