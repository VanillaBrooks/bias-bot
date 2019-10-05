
CREATE TABLE anime
(
    anime_id SERIAL PRIMARY KEY,
    anime_name VARCHAR(100) NOT NULL UNIQUE
);

CREATE TABLE people
(
    character_id SERIAL UNIQUE PRIMARY KEY,
    anime_id SERIAL REFERENCES anime(anime_id),
    character_name VARCHAR(100) NOT NULL,
    unique(character_name, anime_id)
);

CREATE TABLE users 
(
    user_id SERIAL PRIMARY KEY,
    username VARCHAR(30) NOT NULL UNIQUE,
    discord_id BIGINT NOT NULL UNIQUE
);

CREATE TABLE files
(
    file_id SERIAL PRIMARY KEY,
    link VARCHAR(200) NOT NULL UNIQUE,
    character_id SERIAL REFERENCES people(character_id),
    file_name VARCHAR(70) NOT NULL UNIQUE,
    unique(file_id, character_id)
);

CREATE TABLE rating 
(
    user_id SERIAL REFERENCES users(user_id),
    file_id SERIAL REFERENCES files(file_id),
    score SMALLINT NOT NULL,
    unique(user_id, file_id)
);

