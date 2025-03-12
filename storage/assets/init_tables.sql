CREATE TABLE
    pairs_v2 (
        address BYTEA PRIMARY KEY,
        token0 BYTEA NOT NULL,
        token1 BYTEA NOT NULL
    );