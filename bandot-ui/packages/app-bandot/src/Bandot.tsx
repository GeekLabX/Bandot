// Copyright 2017-2019 @polkadot/app-123code authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import BN from 'bn.js';
import React, { useState } from 'react';
import { Button, InputAddress, InputNumber, InputBalance, TxButton } from '@polkadot/react-components';

import Summary from './Summary';

interface Props {
  accountId?: string | null;
}

export default function Bandot ({ accountId }: Props): React.ReactElement<Props> {
  const [bdtVal, setBdtVal] = useState<BN | undefined | null>(null);
  const [bdtRecvId, setBdtRecvId] = useState<string | null>(null);
  const [pdtVal, setPdtVal] = useState<BN | undefined | null>(null);
  const [pdtRecvId, setPdtRecvId] = useState<string | null>(null);
  const [cpuIndex, setCpuIndex] = useState<BN | undefined | null>(null);
  const [cdpVal, setCdpVal] = useState<BN | undefined | null>(null);
  const [skrPrice, setSkrPrice] = useState<BN | undefined | null>(null);

  return (
    <section>
      <h1>Init CDP</h1>
      <div className='ui--row'>
        <div className='large'>
          <Button.Group>
            <TxButton
              accountId={accountId}
              label='Init Pdot'
              tx='pdot.init'
            />
            <TxButton
              accountId={accountId}
              label='Init Bdt'
              tx='bdt.init'
            />
            <TxButton
              accountId={accountId}
              label='Open CDP'
              tx='cdp.open'
            />
          </Button.Group>
        </div>
        <Summary className='small'>Init Bdt.</Summary>
      </div>

      <div className='ui--row'>
        <div className='large'>
          <InputNumber
            label='skrPrice'
            onChange={setSkrPrice}
          />
          <Button.Group>
            <TxButton
              accountId={accountId}
              label='skrPrice'
              params={[skrPrice]}
              tx='cdp.updateSkrPrice'
            />
          </Button.Group>
        </div>
        <Summary className='small'>skr Price.</Summary>
      </div>

      <div className='ui--row'>
        <div className='large'>
          {`user: ${accountId}`}
          <InputNumber
            label='myIndex'
            onChange={setCpuIndex}
          />
          <InputBalance
            label='balance'
            onChange={setCdpVal}
          />
          <Button.Group>
            <TxButton
              accountId={accountId}
              label='Draw'
              params={[cpuIndex, cdpVal]}
              tx='cdp.draw'
            />
            <TxButton
              accountId={accountId}
              label='Lock'
              params={[cpuIndex, cdpVal]}
              tx='cdp.lock'
            />
            <TxButton
              accountId={accountId}
              label='Free'
              params={[cpuIndex, cdpVal]}
              tx='cdp.free'
            />
            <TxButton
              accountId={accountId}
              label='Wipe'
              params={[cpuIndex, cdpVal]}
              tx='cdp.wipe'
            />
          </Button.Group>
        </div>
      </div>

      <h1>Bdt</h1>
      <div className='ui--row'>
        <div className='large'>

          <InputAddress
            className='medium'
            label='to Account'
            onChange={setBdtRecvId}
            type='account'
          />
          <InputBalance
            label='balance'
            onChange={setBdtVal}
          />
          <Button.Group>
            <TxButton
              accountId={accountId}
              label='Mint'
              params={[bdtRecvId, bdtVal]}
              tx='bdt.mint'
            />

            <TxButton
              accountId={accountId}
              label='Burn'
              params={[bdtRecvId, bdtVal]}
              tx='bdt.burn'
            />

            <TxButton
              accountId={accountId}
              label='Tranfer'
              params={[bdtRecvId, bdtVal]}
              tx='bdt.transfer'
            />   
          </Button.Group>
        </div>
        <Summary className='small'>Bdt.</Summary>
      </div>

      <h1>Pdot</h1>

      <div className='ui--row'>
        <div className='large'>

          <InputAddress
            className='medium'
            label='to Account'
            onChange={setPdtRecvId}
            type='account'
          />
          <InputBalance
            label='balance'
            onChange={setPdtVal}
          />
          <Button.Group>
            <TxButton
              accountId={accountId}
              label='Mint'
              params={[pdtRecvId, pdtVal]}
              tx='pdot.mint'
            />

            <TxButton
              accountId={accountId}
              label='Burn'
              params={[pdtRecvId, pdtVal]}
              tx='pdot.burn'
            />

            <TxButton
              accountId={accountId}
              label='Tranfer'
              params={[pdtRecvId, pdtVal]}
              tx='pdot.transfer'
            />   
          </Button.Group>
        </div>
        <Summary className='small'>Pdot.</Summary>
      </div>

      {/* <div className='ui--row'>
        <div className='large'>
          <InputAddress
            label='recipient address for this transfer'
            onChange={setRecipientId}
            type='all'
          />
          <InputBalance
            label='amount to transfer'
            onChange={setAmount}
          />
          <Button.Group>
            <TxButton
              accountId={accountId}
              icon='send'
              label='make transfer'
              params={[recipientId, amount]}
              tx='balances.transfer'
            />
          </Button.Group>
        </div>
        <Summary className='small'>Make a transfer from any account you control to another account. Transfer fees and per-transaction fees apply and will be calculated upon submission.</Summary>
      </div> */}
    </section>
  );
}
