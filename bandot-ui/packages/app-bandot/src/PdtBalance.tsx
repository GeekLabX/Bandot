/* eslint-disable @typescript-eslint/camelcase */
// Copyright 2017-2019 @polkadot/react-query authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import { BareProps, CallProps } from '@polkadot/react-api/types';
import { AccountId, AccountIndex, Address } from '@polkadot/types/interfaces';

import React from 'react';
import { withCalls, } from '@polkadot/react-api';
import { formatNumber } from '@polkadot/util';
import BN from 'bn.js';


interface Props extends BareProps, CallProps {
  pdot_balanceOf?: BN;
  label?: React.ReactNode;
  children?: React.ReactNode;
  params?: AccountId | AccountIndex | Address | string | Uint8Array | null;
}

function BalanceDisplay ({ children, className, label='', style, pdot_balanceOf }: Props): React.ReactElement<Props> {
  return (
    <div className={className}>
      {label}{
        pdot_balanceOf
          ? formatNumber(pdot_balanceOf)
          : '0'
      }{children}
    </div>
  );
}

export default withCalls<Props>(
  ['query.pdot.balanceOf', { paramName: 'params' }]
)(BalanceDisplay);

