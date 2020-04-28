-- Your SQL goes here
ALTER TABLE users
    ADD COLUMN salt VARCHAR(36) NOT NULL DEFAULT '' AFTER password;
