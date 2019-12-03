/* eslint-disable @typescript-eslint/camelcase */
// Copyright 2017-2019 @polkadot/app-123code authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import { AccountId } from '@polkadot/types/interfaces';
import { BareProps, I18nProps } from '@polkadot/react-components/types';

import BN from 'bn.js';
import React from 'react';
import { withCalls } from '@polkadot/react-api';
import { Bubble } from '@polkadot/react-components';
import { formatBalance, formatNumber } from '@polkadot/util';

import translate from './translate';

interface Props extends BareProps, I18nProps {
  bdt_owner?: AccountId;
  bdt_circulation?: BN;
  pdot_circulation?: BN;
  cdp_allCupsCount?: BN;
  cdp_minCollaterlizationRatio?: BN;
}

function SummaryBar ({ bdt_owner, bdt_circulation, pdot_circulation, 
  cdp_allCupsCount, cdp_minCollaterlizationRatio }: Props): React.ReactElement<Props> {

  return (
    <summary>
      <div>
        <Bubble icon='tty' label='Owner'>
          {bdt_owner} 
        </Bubble> 
        <Bubble icon='chain' label='Bdt Circulation'>
          {formatNumber(bdt_circulation)} 
        </Bubble>
        <Bubble icon='chain' label='Pdot Circulation'>
          {formatNumber(pdot_circulation)} 
        </Bubble>
        <br />
        <Bubble icon='chain' label='AllCupsCount'>
          {formatNumber(cdp_allCupsCount)} 
        </Bubble>

        <Bubble icon='chain' label='Collaterlization'>
          {formatNumber(cdp_minCollaterlizationRatio)} 
        </Bubble>
      </div>
    </summary>
  );
}

// inject the actual API calls automatically into props
export default translate(
  withCalls<Props>(
    'query.bdt.owner',
    'query.bdt.circulation',
    'query.pdot.circulation',
    'query.cdp.allCupsCount',
    'query.cdp.minCollaterlizationRatio'
  )(SummaryBar)
);
