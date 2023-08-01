CREATE TABLE IF NOT EXISTS User (
    id INTEGER,
    login NOT NULL UNIQUE,
    PRIMARY KEY (id),
);

CREATE TABLE IF NOT EXISTS Story (
    id INTEGER,
    title TEXT NOT NULL,
   	creator INTEGER NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (creator)
      REFERENCES User (id)
);

INSERT INTO User (login) VALUES('jankohrasko');
INSERT INTO User (login) VALUES('zarosysatravakosi');

INSERT INTO Story (title, creator) VALUES ('jankovo ihryste', 1);
INSERT INTO Story (title, creator) VALUES ('kedy sa kosi trava', 2);
INSERT INTO Story (title, creator) VALUES ('co sa kosi za rosy', 2);
