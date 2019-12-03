// Copyright 2017-2019 @polkadot/app-123code authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import React, { useEffect, useState } from 'react';
import styled from 'styled-components';
import { Bubble, InputAddress, InputNumber } from '@polkadot/react-components';
// import { AccountIndex, Balance, Nonce } from '@polkadot/react-query';
import BdtBalance from './BdtBalance'
import PdtBalance from './PdtBalance'

import UserAssets from './UserAssets'
import UserIndex from './UserIndex'
import BN from 'bn.js';


interface Props {
  className?: string;
  onChange: (accountId: string | null) => void;
}

function AccountSelector ({ className, onChange }: Props): React.ReactElement<Props> {
  const [accountId, setAccountId] = useState<string | null>(null);
  const [myIndex, setMyIndex] = useState<BN | undefined | null>(null);


  useEffect((): void => onChange(accountId), [accountId]);

  return (
    <section className={`template--AccountSelector ${className}`}>
      <div className="ui--row">
        <InputAddress
          className='medium'
          label='my default account'
          onChange={setAccountId}
          type='account'
        />

        <div className='medium'>
          <Bubble color='yellow' icon='adjust' label='pdot'>
            <PdtBalance params={accountId} />
          </Bubble>

          <Bubble color='yellow' icon='adjust' label='bdt'>
            <BdtBalance params={accountId} />
          </Bubble>
          
          <Bubble color='yellow' icon='adjust' label='cdp'>
            <UserAssets account={accountId} />
          </Bubble>
        </div>
      </div>
      <div className='ui--row'>
        <InputNumber
          label='MyIndex'
          onChange={setMyIndex}
        />
        <div className='large'>
        <Bubble color='yellow' icon='adjust' label='cdp index:'>
          <UserIndex params={[[accountId, myIndex]]} />
        </Bubble>
        </div>
        
      </div>

    </section>
  );
}

export default styled(AccountSelector)`
  align-items: flex-end;

  .summary {
    text-align: center;
  }
`;
