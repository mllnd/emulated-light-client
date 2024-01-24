# Restaking

The high level flow of the program is given in the image below.

![Flow of restaking](./restaking-flow.png)

## Accounts

- Vaults: The vaults are created for each whitelisted token. Vaults
  are token accounts. The authority of the account is a PDA which
  means the program controls the vault and any debit from the vault
  has to go through the smart contract.

- Receipt Token Mint: The receipt token mint is a NFT which is the
  seed for the PDA storing information about stake amout, validator
  and rewards. For more information, refer: 
  https://docs.google.com/document/d/1qXEvgYL6MrBQEbt_oFc8NW2znkOApevqz9u7yyA-cgw/edit?usp=sharing

- Staking Params: This is a PDA which stores the staking parameters
  and also is the authority to `Receipt Token Mint` and `Vaults`.

- Vault Params: PDA which stores the vault params which are stake time
  and service for which it is staked along with when rewards were
  claimed.

## Instructions

When the contract is deployed, the `initialize` method is called where
the whitelisted tokens, admin key and the rewards
token mint is set. Initially the `guest_chain_initialization` is set to
false. Any update to the staking paramters can only be
done by the admin key. A token account is also created for the
rewards token mint which would distribute the rewards. Since the
authority is PDA, any debit from the account will happen only through
the contract (only in `claim` method for now). After that the users
can start staking.

- `Deposit`: User can stake any of the whitelisted token. The tokens
  are stored in the vault and receipt tokens are minted for the user.
  A CPI (cross program invocation) call is made to the guest chain
  program where the stake is updated for the validator specified.

- `Withdraw`: Users can only withdraw their tokens after the bounding
  period. When user wants to withdraw the tokens, the rewards and the
  final stake amount is fetched from the guest chain. The receipt
  tokens are burnt and the rewards are returned to the user from the
  vault. A CPI call is made to the guest chain to update the stake
  accordingly.

- `Claim Rewards`: Users can claim rewards without withdrawing their
  stake. They would have to have to own the non fungible receipt
  token to be eligible for claiming rewards.

- `Set Service`: Once the bridge is live, users who had deposited before
  can call this method to delegate their stake to the validator. Users
  cannot withdraw or claim any rewards until they delegate their stake
  to the validator. But this method wont be needed after the bridge is
  live and would panic if called otherwise.

- `Update Guest chain Initialization`: The admin would call this method
  when the bridge is up and running. This would set `guest_chain_program_id`
  with the specified program ID which would allow to make CPI calls during 
  deposit and set stake to validator. 

- `Update token Whitelist`: The admin can update the token whitelist.
  Only callable by admin set during `initialize` method.

- `Withdraw Reward Funds`: This method is only callable by admin to
  withdraw all the funds from the reward token account. This is a
  safety measure so it should be called only during emergency.

## Note

- Since the rewards are not implemented yet on the `Guest Blockchain`,
  a nil value is returned for now.

- Oracle interface is yet to be added to fetch the current price of 
  staked tokens as well as the governance token in the `Guest Blockchain`.

- Users who have deposited before the `guest chain` is initialized can choose
  the validator in one of three ways(Yet to be implemented):
  - Choose a validator randomly
  - Choose a validator from the list of top 10 validators chosen by us
  - Choose a particular validator.
  