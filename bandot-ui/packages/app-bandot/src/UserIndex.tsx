/* eslint-disable @typescript-eslint/camelcase */
// Copyright 2017-2019 @polkadot/react-query authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import { BareProps, CallProps } from '@polkadot/react-api/types';
import { withCalls } from '@polkadot/react-api';

import React, { useState } from 'react';
import { AccountId, AccountIndex, Address } from '@polkadot/types/interfaces';

import { isNull, isUndefined, formatNumber } from '@polkadot/util';
import {Cup} from './Cup'
import BN from 'bn.js';

interface Props extends BareProps, CallProps {
  label?: React.ReactNode;
  children?: React.ReactNode;
  params?: any[];
  index?: BN| number | null;
  cup?: Cup | null;
}

function UserIndex ({ children, className, label='', params, index, cup }: Props): React.ReactElement<Props> {
  console.log(params);
  
  return (
    <div className={className}>
      {label}{
      //     index && `no ${index}:`
      // }{
          cup && `{lock: ${formatNumber(cup.locked_collaterals)}, 
          debt: ${formatNumber(cup.debts)} tax: ${formatNumber(cup.tax)}}`
      }{children}
    </div>
  );
}

export default withCalls<Props>(
  ['query.cdp.ownedCupsArray', { paramName: 'params', propName: 'index' }],
  ['query.cdp.allCupsArray', { paramName: 'index', propName: 'cup' }],
)(UserIndex);

