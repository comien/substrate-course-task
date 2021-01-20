import React from 'react';
import { Button, Card, Grid, Message, Modal, Form, Label, CardContent } from 'semantic-ui-react';

import KittyAvatar from './KittyAvatar';
import { TxButton } from './substrate-lib/components';

// --- About Modal ---

const TransferModal = props => {
  const {
    kitty,
    accountPair,
    setStatus
  } = props;
  const [open, setOpen] = React.useState(false);
  const [formValue, setFormValue] = React.useState({});

  const formChange = key => (ev, el) => {
    /* TODO: 加代码 */
    setFormValue(data => ({
      ...data,
      [key]: el.value
    }));
  };

  const confirmAndClose = (unsub) => {
    unsub();
    setOpen(false);
  };

  return <Modal onClose={() => setOpen(false)} onOpen={() => setOpen(true)} open={open}
                trigger={<Button basic color='blue'>转让</Button>}>
    <Modal.Header>毛孩转让</Modal.Header>
    <Modal.Content><Form>
      <Form.Input fluid label='毛孩 ID' readOnly value={kitty.id}/>
      <Form.Input fluid label='转让对象' placeholder='对方地址' onChange={formChange('target')}/>
    </Form></Modal.Content>
    <Modal.Actions>
      <Button basic color='grey' onClick={() => setOpen(false)}>取消</Button>
      <TxButton
        accountPair={accountPair} label='确认转让' type='SIGNED-TX' setStatus={setStatus}
        onClick={confirmAndClose}
        attrs={{
          palletRpc: 'kittiesModule',
          callable: 'transfer',
          inputParams: [formValue.target, kitty.id],
          paramFields: [true, true]
        }}
      />
    </Modal.Actions>
  </Modal>;
};

// --- About Kitty Card ---

const KittyCard = props => {
  /*
    TODO: 加代码。这里会 UI 显示一张 `KittyCard` 是怎么样的。这里会用到：
    ```
    <KittyAvatar dna={dna} /> - 来描绘一只猫咪
    <TransferModal kitty={kitty} accountPair={accountPair} setStatus={setStatus}/> - 来作转让的弹出层
    ```
  */

  const {
    kitty,
    owner,
    price,
    accountPair,
    setStatus
  } = props;
  let dna = kitty.dna.toString().replace('0x', '');
  let message = '';
  let is_owner = accountPair.address === owner + '';
  return (
    <Grid.Column width={4}>
      <Card>
        <Card.Content>
          <Card.Header>ID：{kitty.id}</Card.Header>
          <div>{is_owner ? '属于本人' : ''}</div>
          <KittyAvatar dna={str2Bytes(dna)}/>
          <Card.Description>
            <div style={{
              overflow: 'hidden',
              wordWrap: 'break-word'
            }}>DNA：{kitty.dna.toString()}</div>
            <div style={{
              overflow: 'hidden',
              wordWrap: 'break-word'
            }}>铲屎官：{owner + ''}</div>
          </Card.Description>
        </Card.Content>
        <Card.Content>
          <TransferModal kitty={kitty} is_owner={is_owner} accountPair={accountPair} setStatus={setStatus}/>
        </Card.Content>
      </Card>
    </Grid.Column>
  );
};

function str2Bytes (str) {
  let pos = 0;
  let len = str.length;
  if (len % 2 != 0) {
    return null;
  }

  len /= 2;
  let hexA = new Array();
  for (let i = 0; i < len; i++) {
    let s = str.substr(pos, 2);
    let v = parseInt(s, 16);
    hexA.push(v);
    pos += 2;
  }
  return hexA;
}

const KittyCards = props => {
  const {
    kitties,
    accountPair,
    setStatus,
    kittyOwners,
    kittyPrices
  } = props;

  /* TODO: 加代码。这里会枚举所有的 `KittyCard` */
  return (
    <Grid>
      {kitties.map((kitty, index) => <KittyCard key={index} kitty={kitty} owner={kittyOwners[index]}
                                                price={kittyPrices[index]} accountPair={accountPair}
                                                setStatus={setStatus}/>)}
    </Grid>
  );
};

export default KittyCards;
