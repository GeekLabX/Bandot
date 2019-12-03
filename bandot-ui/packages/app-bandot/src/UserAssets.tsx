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
  account?: AccountId | AccountIndex | Address | string | Uint8Array | null;
  count?: BN| number | null;
  indics?: [] | null;
  cups?: Cup[] | null;
  index?: BN| number | null;
  cup?: Cup | null;
}

function UserAssets ({ children, className, label='', account, count }: Props): React.ReactElement<Props> {

  return (
    <div className={className}>
      {label}{
          count && formatNumber(count)
      }{children}
    </div>
  );
}

export default withCalls<Props>(
  ['query.cdp.ownedCupsCount', { paramName: 'account', propName: 'count' }],
)(UserAssets);

