CREATE TABLE IF NOT EXISTS user (
    id INTEGER NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    salt TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS access_token (
    user INTEGER NOT NULL,
    token TEXT NOT NULL,
    valid_to INTEGER NOT NULL,
    PRIMARY KEY (user, token),
    FOREIGN KEY (user)
      REFERENCES user (id)
);

CREATE TABLE IF NOT EXISTS story (
    id INTEGER NOT NULL,
    title TEXT NOT NULL,
   	creator INTEGER NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (creator)
      REFERENCES user (id)
);

INSERT INTO User (username, password, salt) VALUES('jankohrasko', 'e6c3da5b206634d7f3f3586d747ffdb36b5c675757b380c6a5fe5c570c714349', 'salt1'); --pass1
INSERT INTO User (username, password, salt) VALUES('zarosysatravakosi', '1ba3d16e9881959f8c9a9762854f72c6e6321cdd44358a10a4e939033117eab9', 'salt2'); --pass2

INSERT INTO Story (title, creator) VALUES ('jankovo ihryste', 1);
INSERT INTO Story (title, creator) VALUES ('kedy sa kosi trava', 2);
INSERT INTO Story (title, creator) VALUES ('co sa kosi za rosy', 2);
