-- Add migration script here
create table users (
  id serial primary key,
  username text not null unique,
  email text not null unique,
  created_at timestamp default now()
);
