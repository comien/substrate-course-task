import React, { useEffect, useState } from 'react';
import { Form, Grid } from 'semantic-ui-react';

import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';

import KittyCards from './KittyCards';

export default function Kitties (props) {
  const {
    api,
    keyring
  } = useSubstrate(); // substrate api
  const { accountPair } = props;

  const [kittyCnt, setKittyCnt] = useState(0);
  const [kittyDNAs, setKittyDNAs] = useState([]);
  const [kittyOwners, setKittyOwners] = useState([]);
  const [kittyPrices, setKittyPrices] = useState([]);
  const [kitties, setKitties] = useState([]);
  const [status, setStatus] = useState('');

  const fetchKittyCnt = () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
    let unsubscribe;
    api.query.kittiesModule.kittiesCount(count => {
      setKittyCnt(count.toNumber());
    }).then(unsub => {
      unsubscribe = unsub;
    }).catch(console.error);
    return () => unsubscribe && unsubscribe();
  };

  const fetchKitties = () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
    let unsubscribe;
    let keys = [];
    for (let i = 0; i < kittyCnt; i++) {
      keys.push(i);
    }
    api.query.kittiesModule.kitties.multi(keys, (data) => {
      const kitty = data.map((dna, index) => {
        return {
          dna: dna,
          id: index,
          is_owner: true
        };
      });
      setKittyDNAs(kitty.dna);
      setKitties(kitty);
    }).then(unsub => {
      unsubscribe = unsub;
    }).catch(console.error);
    return () => unsubscribe && unsubscribe();
  };

  const fetchKittyPrices = () => {
    let unsubscribe;
    let keys = [];
    for (let i = 0; i < kittyCnt; i++) {
      keys.push(i);
    }
    api.query.kittiesModule.kittyPrices.multi(keys, (data) => {
      setKittyPrices(data);
    }).then(unsub => {
      unsubscribe = unsub;
    }).catch(console.error);
    return () => unsubscribe && unsubscribe();
  };

  const populateKitties = () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
    let unsubscribe;
    let keys = [];
    for (let i = 0; i < kittyCnt; i++) {
      keys.push(i);
    }
    api.query.kittiesModule.kittyOwners.multi(keys, (data) => {
      setKittyOwners(data);
    }).then(unsub => {
      unsubscribe = unsub;
    }).catch(console.error);
    return () => unsubscribe && unsubscribe();
  };

  useEffect(fetchKittyCnt, [api, keyring]);
  useEffect(fetchKitties, [api, kittyCnt]);
  useEffect(fetchKittyPrices, [api, kittyCnt]);
  useEffect(populateKitties, [kittyDNAs, kittyOwners]);

  return <Grid.Column width={16}>
    <h1>小毛孩</h1>
    <KittyCards kitties={kitties} accountPair={accountPair} kittyOwners={kittyOwners} kittyPrices={kittyPrices} setStatus={setStatus}/>
    <Form style={{ margin: '1em 0' }}>
      <Form.Field style={{ textAlign: 'center' }}>
        <TxButton
          accountPair={accountPair} label='创建小毛孩' type='SIGNED-TX' setStatus={setStatus}
          attrs={{
            palletRpc: 'kittiesModule',
            callable: 'create',
            inputParams: [],
            paramFields: []
          }}
        />
      </Form.Field>
    </Form>
    <div style={{ overflowWrap: 'break-word' }}>{status}</div>
  </Grid.Column>;
}
