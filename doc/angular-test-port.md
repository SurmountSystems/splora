# Angular test port inventory

This is an inventory of the Mempool Angular Cypress end-to-end tests and two Lightning component specs, classified for this electrs indexer. The explorer is not finished. Signet is not its own chain here. This product redirects `/signet` to `/mutinynet`. Lightning is not planned because there is no Lightning API.

The nine Cypress files live under `ref/mempool/frontend/cypress/e2e/` in network folders, not as the flat names `liquid.cy.ts` and so on. Both Lightning specs live under `ref/mempool/frontend/src/app/lightning/channel/`. There are 173 source `it(` lines in the Cypress files and 2 in the Lightning specs. Two mainnet search `it` calls sit inside `forEach` over two terms each, so those two source lines are four cases. This inventory has one row per case, 177 rows.

Each row has exactly one status: `ported`, `later`, or `no backend`.

## Route evidence from src/rest.rs

`handle_request` in `src/rest.rs` registers these electrs paths, among others:

- `GET /blocks`, `GET /blocks/tip/hash`, `GET /blocks/tip/height`, `GET /block-height/:height`
- `GET /block/:hash`, `GET /block/:hash/status`, `GET /block/:hash/txids`, `GET /block/:hash/header`, `GET /block/:hash/raw`, `GET /block/:hash/txid/:index`, `GET /block/:hash/txs/:start`
- `GET /tx/:hash`, `GET /tx/:hash/hex`, `GET /tx/:hash/raw`, `GET /tx/:hash/status`, `GET /tx/:hash/merkle-proof`, `GET /tx/:hash/outspend/:index`, `GET /tx/:hash/outspends`
- `GET /address/:address`, `GET /address/:address/txs`, `GET /address/:address/txs/chain`, `GET /address/:address/txs/mempool`, `GET /address/:address/utxo`, `GET /address-prefix/:prefix`
- `GET /mempool`, `GET /mempool/txids`, `GET /mempool/recent`, `GET /fee-estimates`
- `GET /block-template`, and only when `enable_mining_rest` is on. That path is not `/v1/mining`.

With the `liquid` feature, the same match also registers `GET /assets/registry`, `GET /assets/registry/search`, `GET /assets/registry/:id`, `GET /asset/:id`, `GET /asset/:id/txs`, `GET /asset/:id/txs/chain`, `GET /asset/:id/txs/mempool`, and `GET /asset/:id/supply`.

`src/rest.rs` has no match arm for `/v1/mining`, `/v1/prices`, `/api/v1/statistics`, `/api/v1/replacements`, `/api/v1/tx/:txid/rbf`, `/api/v1/cpfp`, or `/api/v1/liquid/pegs`. A search of that file finds no `v1/mining`, `v1/prices`, or `v1/statistics` string. `GET /api/v1/ws` is the NIP-98 websocket gate. It is not the Mempool dashboard socket.

`src/http_front/mod.rs` returns HTTP 307 from `/signet` to `/mutinynet` and does not open a signet socket. The same router proxies `/testnet4` to the testnet4 network.

Mining, calculator, and fiat cases are `later` because those routes are absent. Graphs cases that open `/graphs` are `later` because that screen loads `/api/v1/statistics`, which is also absent. Replace-by-fee cases are `later` because they assert `/api/v1/replacements`, `/api/v1/tx/:txid/rbf`, or `/api/v1/cpfp`. Block, transaction, address, recent-transaction, signet, testnet4, and Liquid cases that hit the electrs routes above are `ported`. Both Lightning specs are `no backend`.

## ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts

22 rows. 19 ported, 3 later, 0 no backend.

| File | Test | Asserts | Status |
| --- | --- | --- | --- |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | check first mempool block after skeleton loads | After the skeleton leaves, the first projected mempool block link exists on the Liquid dashboard, which this indexer serves from GET /mempool, GET /fee-estimates, and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | load first mempool block after skeleton loads | Clicking the first projected mempool block link leaves the skeleton, and that dashboard is served from GET /mempool, GET /fee-estimates, and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | loads the dashboard | Visiting the Liquid dashboard waits until the skeleton is gone, and that screen uses GET /mempool and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | loads the blocks page | Clicking the blocks button leaves the skeleton, and the blocks list is GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | loads a specific block page | Opening one Liquid block hash scrolls to pagination and leaves the skeleton, using GET /block/:hash and GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | loads the graphs page | Clicking the graphs button opens /graphs, which redirects to mempool statistics at /api/v1/statistics, and src/rest.rs does not register that route. | later |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | loads the graphs page - mobile | On a phone viewport the graphs page has no television-only element, and that page still depends on /api/v1/statistics, which src/rest.rs does not register. | later |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | peg in/peg out / loads peg in addresses | A Liquid transaction page shows the text Peg-in and follows the input link, and the page under test is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | peg in/peg out / loads peg out addresses | A Liquid transaction output link lands on a Bitcoin address page that shows a Liquid Peg Out badge, using GET /tx/:hash and GET /address/:address. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | assets / shows the assets screen | The assets screen shows at least five featured cards, and the list route is GET /assets/registry when the liquid feature is compiled. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | assets / allows searching assets | Typing Tether USD opens one typeahead window, and asset search is GET /assets/registry/search when the liquid feature is compiled. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | assets / shows a specific asset ID | Choosing Liquid AUD from the typeahead opens that asset, using GET /assets/registry/search and GET /asset/:id when the liquid feature is compiled. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | unblinded TX / should not show an unblinding error message for regular txs | A normal Liquid transaction page does not show an unblinding error, and the transaction is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | unblinded TX / show unblinded TX | Blinding data in the URL hash reveals LBTC amounts on inputs and outputs, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | unblinded TX / show empty unblinded TX | An empty blinded hash leaves inputs and outputs labeled Confidential, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | unblinded TX / show invalid unblinded TX hex | A short blinded hash shows the invalid-hex unblinding error, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | unblinded TX / show first unblinded vout | Blinding data for the first output shows 0.00100000 LBTC in an asset box, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | unblinded TX / show second unblinded vout | Blinding data for the second output shows 0.02364760 LBTC in an asset box, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | unblinded TX / show invalid error unblinded TX | Almost-valid blinding data still shows the invalid blinding-data error, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | unblinded TX / shows asset peg in/out and burn transactions | An asset page renders peg and burn rows that are not asset boxes, using GET /asset/:id and GET /asset/:id/txs when the liquid feature is compiled. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | unblinded TX / prevents regressing issue #644 | Opening that transaction waits until the skeleton is gone, and the transaction is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | Tests cannot be run on the selected BASE_MODULE ${baseModule} | This case is skipped, it runs only when BASE_MODULE is not liquid, and it asserts no indexer route. | later |

## ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts

20 rows. 19 ported, 1 later, 0 no backend.

The two tests whose titles say graphs do not open `/graphs`. They only look at the dashboard, so they are ported. The Liquid file above does open `/graphs`, and those two rows are later.

| File | Test | Asserts | Status |
| --- | --- | --- | --- |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | check first mempool block after skeleton loads | After the skeleton leaves, the first projected mempool block link exists on the Liquid testnet dashboard, which this indexer serves from GET /mempool, GET /fee-estimates, and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | loads the dashboard | Visiting the Liquid testnet path waits until the skeleton is gone, and that screen uses GET /mempool and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | loads the dashboard with no scrollbars on mobile | This case is skipped, and if it ran it would require a phone viewport of the Liquid testnet dashboard to have no horizontal scrollbar, using GET /mempool and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | loads the blocks page | The test visits the dashboard and checks that the blocks button is present, and it does not open the blocks list, whose route would be GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | loads a specific block page | Opening one Liquid testnet block hash scrolls to pagination and leaves the skeleton, using GET /block/:hash and GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | loads the graphs page | The test visits the dashboard and checks that the graphs button is present, and it does not open /graphs, so the screen under test is still GET /mempool and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | loads the graphs page - mobile | On a phone viewport the dashboard has no television-only element, and the test never opens /graphs, so the screen is GET /mempool and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | renders unconfidential transactions correctly on mobile | This case is skipped, and if it ran it would open a transaction with blinding data on a phone and require no horizontal scrollbar, using GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | assets / allows searching assets | Typing Tether USD opens one typeahead window, and asset search is GET /assets/registry/search when the liquid feature is compiled. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | assets / shows a specific asset ID | Choosing Liquid CAD from the typeahead opens that asset, using GET /assets/registry/search and GET /asset/:id when the liquid feature is compiled. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | unblinded TX / should not show an unblinding error message for regular txs | A normal Liquid testnet transaction page does not show an unblinding error, and the transaction is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | unblinded TX / show unblinded TX | Blinding data in the URL hash reveals tLBTC amounts on inputs and outputs, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | unblinded TX / show empty unblinded TX | An empty blinded hash leaves inputs and outputs labeled Confidential, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | unblinded TX / show invalid unblinded TX hex | A short blinded hash shows the invalid-hex unblinding error, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | unblinded TX / show first unblinded vout | Blinding data shows 0.00099729 tLBTC on the first output, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | unblinded TX / show second unblinded vout (asset) | Blinding data shows the second output as an asset box containing 0 TEST, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | unblinded TX / should link to the asset page from the unblinded tx | Clicking the second output amount opens that asset URL, using GET /tx/:hash and GET /asset/:id when the liquid feature is compiled. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | unblinded TX / show invalid error unblinded TX | Almost-valid blinding data still shows the invalid blinding-data error, and the transaction body is GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | unblinded TX / shows asset peg in/out and burn transactions | An asset page renders peg and burn rows that are not asset boxes, using GET /asset/:id and GET /asset/:id/txs when the liquid feature is compiled. | ported |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | Tests cannot be run on the selected BASE_MODULE ${baseModule} | This case is skipped, it runs only when BASE_MODULE is not liquid, and it asserts no indexer route. | later |

## ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts

25 rows. 0 ported, 25 later, 0 no backend.

Every case depends on a fiat conversion feed. `src/rest.rs` has no `/v1/prices` route and no other price route.

| File | Test | Asserts | Status |
| --- | --- | --- | --- |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | page load and initial state / loads the calculator page with heading and form | The page shows the Calculator heading and the fiat, BTC, and sats inputs after the mocked price feed, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | page load and initial state / displays the mocked conversion rate in .symbol | The symbol text includes the mocked USD rate, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | page load and initial state / shows copy buttons for each input | Three clipboard controls are visible, and the page still depends on a price feed that src/rest.rs does not serve. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | page load and initial state / shows fiat price display and bitcoin visual | The fiat-price timestamp and the bitcoin visual are visible, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | page load and initial state / shows input labels for currency, BTC, and sats | The BTC and sats input labels are visible, and the calculator still depends on prices that src/rest.rs does not serve. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | default values / displays 1 BTC with correct sats and fiat | The form shows 1 BTC, 100000000 sats, and the mocked USD amount, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | bitcoin input updates fiat and sats / updates fiat and sats when entering 0.33 BTC | Entering 0.33 BTC sets sats to 33000000 and fiat to the mocked USD conversion, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | bitcoin input updates fiat and sats / updates fiat and sats when entering 1 sat (0.00000001 BTC) | Entering one sat sets sats to 1 and updates the fiat field, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | fiat input updates BTC and sats / updates BTC and sats when entering fiat value | Entering a fiat amount updates BTC and sats from the mocked USD rate, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | satoshis input updates BTC and fiat / updates BTC and fiat when entering 10000 sats | Entering 10000 sats sets BTC to 0.00010000 and the mocked USD fiat amount, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | input sanitization / normalizes comma to dot in fiat input | Typing a comma in the fiat field is stored as a dot, and the calculator still depends on prices that src/rest.rs does not serve. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | input sanitization / limits BTC to 8 decimals | A BTC entry keeps at most eight decimal places, and the calculator still depends on prices that src/rest.rs does not serve. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | input sanitization / strips decimals from satoshis input | A decimal satoshi entry is stored without a dot, and the calculator still depends on prices that src/rest.rs does not serve. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | max supply (21M BTC) / shows warning when entering 21M BTC | Entering 21000000 BTC shows the max-supply warning and keeps that cap, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | max supply (21M BTC) / caps values at max supply | Entering 25000000 BTC is capped at 21000000 and shows the warning, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | clipboard buttons / copy buttons exist and are visible | Three clipboard buttons are visible, and the calculator still depends on prices that src/rest.rs does not serve. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | responsive viewports / calculator is usable on desktop | The BTC input stays usable on a desktop viewport, and the calculator still depends on prices that src/rest.rs does not serve. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | responsive viewports / calculator is usable on mobile | The BTC, fiat, and sats inputs stay visible on a phone viewport, and the calculator still depends on prices that src/rest.rs does not serve. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | loading state / shows calculator form after price feed loads | The waiting-for-price message is gone and the BTC input is visible, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | JPY currency / displays JPY conversion rate in .symbol | Selecting JPY shows the mocked yen rate in the symbol and the yen fiat text, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | JPY currency / displays 1 BTC with correct sats and fiat in JPY | With JPY selected, 1 BTC still maps to 100000000 sats and the mocked yen amount, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | JPY currency / updates fiat and sats when entering 0.5 BTC in JPY | Entering 0.5 BTC sets sats to 50000000 and the mocked yen fiat amount, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | JPY currency / updates BTC and sats when entering fiat value in JPY | Entering a yen amount updates BTC and sats from the mocked yen rate, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | JPY currency / updates BTC and fiat when entering 10000 sats in JPY | Entering 10000 sats sets BTC to 0.00010000 and the mocked yen fiat amount, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | Tests cannot be run on the selected BASE_MODULE ${calculatorBaseModule} | This case is skipped, it runs only when BASE_MODULE is not mempool, and it asserts no indexer route. | later |

## ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts

18 rows. 0 ported, 18 later, 0 no backend.

Every case formats USD or JPY amounts. `src/rest.rs` has no price route. Block and transaction pages also use electrs routes, but the assertion is the fiat formatting, so the status is later.

| File | Test | Asserts | Status |
| --- | --- | --- | --- |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Dashboard / USD formatting / displays USD values with correct currency symbol | A dashboard fiat cell includes a dollar sign after USD is selected, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Dashboard / USD formatting / displays USD values with proper decimal format | A dashboard fiat cell matches a dollar amount with optional cents, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Dashboard / JPY formatting / displays JPY values with yen symbol | A dashboard fiat cell includes a yen sign after JPY is selected, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Dashboard / JPY formatting / displays JPY values without decimal places | A dashboard yen amount has no trailing decimal, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Dashboard / JPY formatting / formats all JPY values correctly without decimals | Every dashboard cell that contains a yen sign has no decimal fraction, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Dashboard / currency switching / correctly formats when switching from USD to JPY | Switching the dashboard from USD to JPY replaces the dollar sign with a yen amount that has no trailing decimal, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Dashboard / currency switching / correctly formats when switching from JPY back to USD | Switching the dashboard from JPY back to USD shows a dollar sign again, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Transaction Page / displays USD fiat values with dollar symbol | The transaction details and the transaction list show a dollar sign in fiat mode, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Transaction Page / displays JPY fiat values without decimals | The transaction details and the transaction list show a yen amount with no decimal fraction, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Block Page / displays USD fiat values in block reward with dollar symbol | Every visible fiat element on the block page includes a dollar sign, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Block Page / displays JPY fiat values without decimals | Every visible fiat element on the block page is a yen amount with no decimal fraction, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Address Page / displays USD fiat values with dollar symbol | Address amount fiat labels include a dollar sign, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Address Page / displays JPY fiat values without decimals | The first address fiat label is a yen amount with no decimal fraction, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Calculator Page / displays USD price with dollar symbol | The calculator symbol includes a dollar sign when USD is selected, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Calculator Page / displays JPY price without decimals after switching currency | After switching to JPY and entering 1 BTC, the symbol shows a yen sign and the fiat field has no trailing decimal, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Mempool Block Tooltip / displays USD fiat values in mempool block | Fiat text inside the first mempool block includes a dollar sign, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Mempool Block Tooltip / displays JPY fiat values in mempool block | Fiat text inside the first mempool block is a yen amount with no trailing decimal, and src/rest.rs has no price route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | Tests cannot be run on the selected BASE_MODULE ${baseModule} | This case is skipped, it runs only when BASE_MODULE is not mempool, and it asserts no indexer route. | later |

## ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts

44 rows. 32 ported, 12 later, 0 no backend.

The partial bech32 and bech32m searches are one source `it` each, inside `forEach` over two spellings. Each spelling is its own row.

| File | Test | Asserts | Status |
| --- | --- | --- | --- |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | check first mempool block after skeleton loads | After the skeleton leaves, the first projected mempool block link exists, and that dashboard is served from GET /mempool, GET /fee-estimates, and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | loads the status screen | The status view shows one mempool block, 22 chain blocks, and footer text for incoming transactions, unconfirmed count, and mempool size, using GET /mempool and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | loads dashboard, drop websocket and reconnect | This case is skipped, and if it ran it would drop the Mempool dashboard socket and expect a reconnect, which is not the NIP-98 GET /api/v1/ws gate in src/rest.rs. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | loads the dashboard | Visiting the dashboard waits until the skeleton is gone, and that screen uses GET /mempool and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | check op_return tx tooltip | Hovering the first link in the second transaction row shows a tooltip, and the block page is GET /block/:hash plus GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | check op_return coinbase tooltip | Hovering the same block-page link shows a tooltip for the coinbase row, and the block page is GET /block/:hash plus GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | search / allows searching for partial Bitcoin addresses | Typing a partial base58 address narrows the dropdown and opens that address without an invalid-address message, using GET /address-prefix/:prefix and GET /address/:address. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | search / allows searching for partial case insensitive bech32m addresses: BC1PQYQS | Typing BC1PQYQS lists ten addresses and opens the matching bc1p address, using GET /address-prefix/:prefix and GET /address/:address. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | search / allows searching for partial case insensitive bech32m addresses: bc1PqYqS | Typing bc1PqYqS lists ten addresses and opens the same bc1p address, using GET /address-prefix/:prefix and GET /address/:address. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | search / allows searching for partial case insensitive bech32 addresses: BC1Q0003 | Typing BC1Q0003 lists ten addresses and opens the matching bc1q address, using GET /address-prefix/:prefix and GET /address/:address. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | search / allows searching for partial case insensitive bech32 addresses: bC1q0003 | Typing bC1q0003 lists ten addresses and opens the same bc1q address, using GET /address-prefix/:prefix and GET /address/:address. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | address highlighting / highlights single input addresses | The address page highlights that address once in the inputs of the first transaction, using GET /address/:address/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | address highlighting / highlights multiple input addresses | The address page highlights that address twice in the inputs of a later transaction, using GET /address/:address/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | address highlighting / highlights both input and output addresses in the same transaction | The address page highlights that address on both the input and output sides of one transaction, using GET /address/:address/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | address highlighting / highlights single output addresses | The address page highlights that address once in the outputs of a transaction, using GET /address/:address/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | address highlighting / highlights multiple output addresses | The address page highlights that address twice in the outputs of a transaction, using GET /address/:address/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | address poisoning / highlights potential address poisoning attacks on outputs, prefix and infix | The transaction page shows two poison alerts and splits lookalike outputs into the expected prefix and infix, using GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | address poisoning / highlights potential address poisoning attacks on inputs and outputs, prefix, infix and postfix | The transaction page shows two poison alerts and splits lookalike addresses into prefix, infix, and postfix, using GET /tx/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks navigation / keyboard events / loads first blockchain block visible and keypress arrow right | Opening the newest visible block and pressing right reveals both block arrows, and the block page is GET /block/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks navigation / keyboard events / loads first blockchain block visible and keypress arrow left | Opening the newest visible block and pressing left sets the title to Next Block, and the block page is GET /block/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks navigation / keyboard events / loads last blockchain block and keypress arrow right | This case is skipped because the comment says the last block does not work with infinite scrolling, and the block page it would open is GET /block/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks navigation / keyboard events / loads genesis block and keypress arrow right | On the genesis block, pressing right shows a next arrow and no previous arrow, and the block page is GET /block/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks navigation / keyboard events / loads genesis block and keypress arrow left | On the genesis block, pressing left shows both arrows, and the block page is GET /block/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks navigation / mouse events / loads first blockchain blocks visible and click on the arrow right | Clicking previous from the newest visible block reveals both arrows, and the block page is GET /block/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks navigation / mouse events / loads genesis block and click on the arrow left | Clicking next from the genesis block reveals both arrows, and the block page is GET /block/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | loads skeleton when changes between networks | Changing from mainnet to signet and back shows the network skeleton, and /signet is an HTTP 307 to /mutinynet rather than its own chain, with chain data from GET /blocks and GET /mempool. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | loads the dashboard with the skeleton blocks | This case is skipped, and if it ran it would show skeleton chain and mempool blocks until a mocked init, using GET /blocks and GET /mempool. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | loads the pools screen | Clicking the pools button opens /mining, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | loads the graphs screen | Clicking the graphs button opens /graphs, which loads /api/v1/statistics, and src/rest.rs does not register that route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | graphs page / check buttons - mobile | On a phone viewport the graphs page shows the fee dropdown and button group, and that page depends on /api/v1/statistics, which src/rest.rs does not register. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | graphs page / check buttons - tablet | On a tablet viewport the graphs page shows the fee dropdown and button group, and that page depends on /api/v1/statistics, which src/rest.rs does not register. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | graphs page / check buttons - desktop | On a desktop viewport the graphs page shows the fee dropdown and button group, and that page depends on /api/v1/statistics, which src/rest.rs does not register. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | loads the api screen | Clicking the docs button waits one second, and that Mempool docs screen is not an electrs block, transaction, address, or recent-transaction route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks / shows empty blocks properly | That block page heading reads 1 transaction, using GET /block/:hash and GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks / expands and collapses the block details | The details button shows and then hides the block details panel, and the block page is GET /block/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks / shows blocks with no pagination | That block heading reads 19 transactions and the first pager has five children, using GET /block/:hash and GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks / supports pagination on the block screen | Clicking the next page changes the transaction header text, using GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks / shows blocks pagination with 5 pages (desktop) | At a wide viewport the first pager has nine children, using GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | blocks / shows blocks pagination with 3 pages (mobile) | At a narrow viewport the first pager has seven children, using GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | RBF transactions / RBF page gets updated over websockets | The replacements page grows from no trees to two trees as socket fixtures arrive, and src/rest.rs has no /api/v1/replacements route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | RBF transactions / shows RBF transactions properly (mobile - details) | On a phone, a details-mode transaction shows a mempool alert after a mocked replace-by-fee event, and src/rest.rs has no /api/v1/tx/:txid/rbf route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | RBF transactions / shows RBF transactions properly (mobile - tracker) | The tracker follows a replaced transaction through confirmation, and the fixtures call /api/v1/tx/:txid/rbf, /api/v1/cpfp, and /api/v1/mining/pools/1w, none of which src/rest.rs registers. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | RBF transactions / shows RBF transactions properly (desktop) | On a desktop transaction page the replace-by-fee alert is visible and does not overlap the confirmations control, and src/rest.rs has no /api/v1/tx/:txid/rbf route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | Tests cannot be run on the selected BASE_MODULE ${baseModule} | This case is skipped, it runs only when BASE_MODULE is not mempool, and it asserts no indexer route. | later |

## ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts

19 rows. 0 ported, 19 later, 0 no backend.

`describe.only('mining graphs')` focuses the seven graph cases when Cypress runs this file, so the miner-page and dashboard-widget cases are excluded at runtime. They are still rows. None of them is `it.skip` except the base-module guard. `src/rest.rs` has no `/v1/mining` route. `GET /block-template` is not that API.

| File | Test | Asserts | Status |
| --- | --- | --- | --- |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Miner page / loads the mining pool page from the dashboard | Clicking a mined block pool badge opens a /mining/pool URL after GET /api/v1/mining/pool, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Miner page / loads the mining pool page from the blocks page | Opening a block from the mining dashboard and clicking the miner badge lands on /mining/pool, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Mining Dashboard Landing page widgets / shows the mempool blocks | The first mempool block on /mining shows fee rate, fee span, total fees, transaction count, and an expected time, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Mining Dashboard Landing page widgets / shows the mined blocks | The newest mined block shows height, fees, transaction count, time, and a pool name, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Mining Dashboard Landing page widgets / shows the reward stats for the last 144 blocks | The reward-stats widget is present, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Mining Dashboard Landing page widgets / shows the difficulty adjustment stats | The difficulty-adjustment widget is present, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Mining Dashboard Landing page widgets / shows the latest blocks | The latest-blocks widget is present, and this is the first of two identical it titles, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Mining Dashboard Landing page widgets / shows the pools pie chart | The pool-distribution widget is present, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Mining Dashboard Landing page widgets / shows the hashrate graph | The hashrate graph widget is present, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Mining Dashboard Landing page widgets / shows the latest blocks (second it) | The latest-blocks widget is present again, and this second identical it still needs /v1/mining, which src/rest.rs does not register. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Mining Dashboard Landing page widgets / shows the latest adjustments | The difficulty-adjustments table is present, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | mining graphs / pools ranking / loads the graph | Visiting /graphs/mining/pools leaves the spinner, this it sits under describe.only, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | mining graphs / pools dominance / loads the graph | Visiting /graphs/mining/pools-dominance leaves the spinner, this it sits under describe.only, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | mining graphs / hashrate & difficulty / loads the graph | Visiting /graphs/mining/hashrate-difficulty leaves the spinner, this it sits under describe.only, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | mining graphs / block fee rates / loads the graph | Visiting /graphs/mining/block-fee-rates leaves the spinner, this it sits under describe.only, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | mining graphs / block fees / loads the graph | Visiting /graphs/mining/block-fees leaves the spinner, this it sits under describe.only, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | mining graphs / block rewards / loads the graph | Visiting /graphs/mining/block-rewards leaves the spinner, this it sits under describe.only, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | mining graphs / block sizes and weights / loads the graph | Visiting /graphs/mining/block-sizes-weights leaves the spinner, this it sits under describe.only, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | Tests cannot be run on the selected BASE_MODULE ${baseModule} | This case is skipped, it runs only when BASE_MODULE is not mempool, and it asserts no indexer route. | later |

## ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts

5 rows. 5 ported, 0 later, 0 no backend.

The file feeds rows through a mocked websocket. The matching electrs route is `GET /mempool/recent`.

| File | Test | Asserts | Status |
| --- | --- | --- | --- |
| ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts | updates the transaction list over time | The recent-transaction list gains a newly sent txid, which this indexer serves at GET /mempool/recent, although the spec feeds rows through a mocked websocket. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts | pauses updates when clicking the pause icon | After pause, a newly sent txid stays out of the list and the visible txids do not change, and the list route is GET /mempool/recent. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts | caps the list when changing the limit to 10 | The list holds 50 rows and then 10 rows after the limit control is clicked, and the list route is GET /mempool/recent. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts | shows the new transaction pill when there are new transactions | Scrolling down and receiving more transactions shows the new-transaction pill, and the list route is GET /mempool/recent. | ported |
| ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts | shows the new transaction pill when there are new transactions and scrolls to the top when clicked | Clicking the new-transaction pill scrolls the window back to the top, and the list route is GET /mempool/recent. | ported |

## ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts

11 rows. 7 ported, 4 later, 0 no backend.

Signet is not its own chain. `src/http_front/mod.rs` returns HTTP 307 from `/signet` to `/mutinynet`. Block cases after that redirect use the electrs block routes. Pools, graphs, and the docs click do not.

| File | Test | Asserts | Status |
| --- | --- | --- | --- |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | loads the dashboard | Visiting /signet waits until the skeleton is gone, and this product returns HTTP 307 from /signet to /mutinynet, whose dashboard uses GET /blocks and GET /mempool. | ported |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | check first mempool block after skeleton loads | After the skeleton leaves, the first projected mempool block link exists, and that dashboard is served from GET /mempool, GET /fee-estimates, and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | loads the dashboard with the skeleton blocks | This case is skipped, and if it ran it would show skeleton blocks on /signet until a mocked init, after the HTTP 307 from /signet to /mutinynet and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | loads the pools screen | Clicking the pools button opens /mining, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | loads the graphs screen | Clicking the graphs button opens /graphs, which loads /api/v1/statistics, and src/rest.rs does not register that route. | later |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | loads the api screen | Clicking the docs button waits one second, and that Mempool docs screen is not an electrs block, transaction, address, or recent-transaction route. | later |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | blocks / shows empty blocks properly | That signet block heading reads 1 transaction, using GET /block/:hash and GET /block/:hash/txs after /signet redirects to /mutinynet. | ported |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | blocks / expands and collapses the block details | The details button shows and then hides the block details panel, and the block page is GET /block/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | blocks / shows blocks with no pagination | That block heading reads 13 transactions and the first pager has five children, using GET /block/:hash and GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | blocks / supports pagination on the block screen | Clicking the next page changes the transaction header text, using GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | Tests cannot be run on the selected BASE_MODULE ${baseModule} | This case is skipped, it runs only when BASE_MODULE is not mempool, and it asserts no indexer route. | later |

## ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts

11 rows. 7 ported, 4 later, 0 no backend.

`src/http_front/mod.rs` proxies `/testnet4` to the testnet4 network. Block and dashboard cases use the electrs routes on that network. Pools, graphs, and the docs click do not.

| File | Test | Asserts | Status |
| --- | --- | --- | --- |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | loads the dashboard | Visiting /testnet4 waits until the skeleton is gone, and the front proxies /testnet4 to the testnet4 network, whose dashboard uses GET /blocks and GET /mempool. | ported |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | check first mempool block after skeleton loads | After the skeleton leaves, the first projected mempool block link exists, and that dashboard is served from GET /mempool, GET /fee-estimates, and GET /blocks. | ported |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | loads the dashboard with the skeleton blocks | This case is skipped, and if it ran it would show skeleton blocks on /testnet4 until a mocked init, using GET /blocks and GET /mempool on the proxied testnet4 network. | ported |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | loads the pools screen | Clicking the pools button opens /mining, and src/rest.rs has no /v1/mining route. | later |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | loads the graphs screen | Clicking the graphs button opens /graphs, which loads /api/v1/statistics, and src/rest.rs does not register that route. | later |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | loads the api screen | Clicking the docs button waits one second, and that Mempool docs screen is not an electrs block, transaction, address, or recent-transaction route. | later |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | blocks / shows empty blocks properly | The genesis block heading reads 1 transaction, using GET /block/:hash and GET /block/:hash/txs on the proxied testnet4 network. | ported |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | blocks / expands and collapses the block details | The details button shows and then hides the block details panel, and the block page is GET /block/:hash. | ported |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | blocks / shows blocks with no pagination | That block heading reads 18 transactions and the first pager has five children, using GET /block/:hash and GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | blocks / supports pagination on the block screen | Clicking the next page changes the transaction header text, using GET /block/:hash/txs. | ported |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | Tests cannot be run on the selected BASE_MODULE ${baseModule} | This case is skipped, it runs only when BASE_MODULE is not mempool, and it asserts no indexer route. | later |

## ref/mempool/frontend/src/app/lightning/channel/channel-box/channel-box.component.spec.ts

1 row. 0 ported, 0 later, 1 no backend.

This indexer has no Lightning API. A Lightning UI is not planned. This row is not ported and not later.

| File | Test | Asserts | Status |
| --- | --- | --- | --- |
| ref/mempool/frontend/src/app/lightning/channel/channel-box/channel-box.component.spec.ts | ChannelBoxComponent / should create | The spec only checks that ChannelBoxComponent constructs, and this indexer has no Lightning API. | no backend |

## ref/mempool/frontend/src/app/lightning/channel/channel-close-box/channel-close-box.component.spec.ts

1 row. 0 ported, 0 later, 1 no backend.

| File | Test | Asserts | Status |
| --- | --- | --- | --- |
| ref/mempool/frontend/src/app/lightning/channel/channel-close-box/channel-close-box.component.spec.ts | ChannelCloseBoxComponent / should create | The spec only checks that ChannelCloseBoxComponent constructs, and this indexer has no Lightning API. | no backend |

## Classification notes

These rows were the least obvious. The status above is the one this inventory uses.

- Liquid testnet `loads the graphs page` and `loads the graphs page - mobile` do not open `/graphs`. They stay on the dashboard, so they are ported. The Liquid file's twins do open `/graphs` and are later.
- Mainnet, signet, and testnet4 `loads the api screen` only click the docs button. They do not need `/v1/mining` or prices. They are later because the docs screen is not an electrs block, transaction, address, or recent-transaction route.
- Mainnet `loads the status screen` is ported from `GET /mempool` and `GET /blocks`. `GET /mempool` backlog stats are count, vsize, total fee, and a fee histogram. They do not include an incoming virtual-byte-per-second rate, which the footer text also checks.
- Mainnet `loads dashboard, drop websocket and reconnect` is later. `GET /api/v1/ws` is a NIP-98 gate, not the Mempool dashboard socket. The case is skipped.
- Replace-by-fee rows open transaction or `/rbf` pages, but the assertions use `/api/v1/replacements`, `/api/v1/tx/:txid/rbf`, or `/api/v1/cpfp`. Those routes are absent, so the rows are later.
- `describe.only('mining graphs')` focuses seven mining graph cases and excludes the other mining describes at runtime. Every mining `it` is still a later row.
- Liquid asset routes exist in `src/rest.rs` only inside `cfg(feature = "liquid")`. Those rows are ported because the match arms are in that file.
- Each `Tests cannot be run on the selected BASE_MODULE` row is an `it.skip` guard. It asserts no route, so it is later.

## Totals

| File | Rows | Ported | Later | No backend |
| --- | --- | --- | --- | --- |
| ref/mempool/frontend/cypress/e2e/liquid/liquid.spec.ts | 22 | 19 | 3 | 0 |
| ref/mempool/frontend/cypress/e2e/liquidtestnet/liquidtestnet.spec.ts | 20 | 19 | 1 | 0 |
| ref/mempool/frontend/cypress/e2e/mainnet/calculator.spec.ts | 25 | 0 | 25 | 0 |
| ref/mempool/frontend/cypress/e2e/mainnet/fiat-currency-formatting.spec.ts | 18 | 0 | 18 | 0 |
| ref/mempool/frontend/cypress/e2e/mainnet/mainnet.spec.ts | 44 | 32 | 12 | 0 |
| ref/mempool/frontend/cypress/e2e/mainnet/mining.spec.ts | 19 | 0 | 19 | 0 |
| ref/mempool/frontend/cypress/e2e/mainnet/recent-transactions.spec.ts | 5 | 5 | 0 | 0 |
| ref/mempool/frontend/cypress/e2e/signet/signet.spec.ts | 11 | 7 | 4 | 0 |
| ref/mempool/frontend/cypress/e2e/testnet4/testnet4.spec.ts | 11 | 7 | 4 | 0 |
| ref/mempool/frontend/src/app/lightning/channel/channel-box/channel-box.component.spec.ts | 1 | 0 | 0 | 1 |
| ref/mempool/frontend/src/app/lightning/channel/channel-close-box/channel-close-box.component.spec.ts | 1 | 0 | 0 | 1 |
| Total | 177 | 89 | 86 | 2 |

Files: 11. Rows: 177. Ported: 89. Later: 86. No backend: 2.

Arithmetic: 89 ported plus 86 later plus 2 no backend equals 177 rows.

The explorer is not finished.
