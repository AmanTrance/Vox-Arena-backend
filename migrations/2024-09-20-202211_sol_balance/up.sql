CREATE TABLE sol_balance(
    balance FLOAT NOT NULL,
    user_id INT PRIMARY KEY NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
