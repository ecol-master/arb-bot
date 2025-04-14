CREATE TABLE
    dexes (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL UNIQUE
    );

CREATE TABLE
    trading_pairs (
        address BYTEA PRIMARY KEY,
        dex_id INT NOT NULL,
        token0 BYTEA NOT NULL,
        token1 BYTEA NOT NULL,
        FOREIGN KEY (dex_id) REFERENCES dexes (id) ON DELETE CASCADE
    );

CREATE TABLE
    token_tickers (token BYTEA PRIMARY KEY, ticker TEXT NOT NULL);

-- Migrate table
-- INSERT INTO trading_pairs (address, dex_id, token0, token1)
-- SELECT p.address, e.id, p.token0, p.token1
-- FROM pairs_v2 p
-- JOIN dexes e ON e.name = 'uniswap_v2';