CREATE SCHEMA intrared;
CREATE TABLE intrared.groups (
  id uuid NOT NULL,
  data jsonb,
  version integer NOT NULL
);
CREATE TABLE intrared.r_role_group (
  id_role uuid NOT NULL,
  id_group uuid NOT NULL
);
CREATE TABLE intrared.r_user_group (
  id_user uuid NOT NULL,
  id_group uuid NOT NULL
);
CREATE TABLE intrared.roles (
  id uuid NOT NULL,
  data jsonb,
  version integer NOT NULL
);
CREATE TABLE intrared.users (
  id uuid NOT NULL,
  data jsonb,
  version integer NOT NULL
);
ALTER TABLE ONLY intrared.groups
ADD
  CONSTRAINT groups_pkey PRIMARY KEY (id);
ALTER TABLE ONLY intrared.r_role_group
ADD
  CONSTRAINT r_role_group_pkey PRIMARY KEY (id_role, id_group);
ALTER TABLE ONLY intrared.r_user_group
ADD
  CONSTRAINT r_user_group_pkey PRIMARY KEY (id_user, id_group);
ALTER TABLE ONLY intrared.roles
ADD
  CONSTRAINT roles_pkey PRIMARY KEY (id);
ALTER TABLE ONLY intrared.users
ADD
  CONSTRAINT users_pkey PRIMARY KEY (id);
CREATE INDEX fki_rg_id_group_fk ON intrared.r_role_group USING btree (id_group);
CREATE INDEX fki_rg_id_role_fk ON intrared.r_role_group USING btree (id_role);
CREATE INDEX fki_ug_id_group_fk ON intrared.r_user_group USING btree (id_group);
CREATE INDEX fki_ug_id_user_fk ON intrared.r_user_group USING btree (id_user);
CREATE INDEX group_data_ix ON intrared.groups USING gin (data);
CREATE UNIQUE INDEX group_id_ix ON intrared.groups USING btree (id);
CREATE UNIQUE INDEX group_name_ix ON intrared.groups USING btree (((data -> 'name' :: text)));
CREATE UNIQUE INDEX role_app_name_ix ON intrared.roles USING btree (
    ((data -> 'app' :: text)),
    ((data -> 'name' :: text))
  );
CREATE INDEX role_data_ix ON intrared.roles USING gin (data);
CREATE UNIQUE INDEX role_id_ix ON intrared.roles USING btree (id);
CREATE INDEX user_data_ix ON intrared.users USING gin (data);
CREATE UNIQUE INDEX user_email_ix ON intrared.users USING btree (((data -> 'email' :: text)));
CREATE UNIQUE INDEX user_id_ix ON intrared.users USING btree (id);
CREATE UNIQUE INDEX user_username_ix ON intrared.users USING btree (((data -> 'user_name' :: text)));
ALTER TABLE ONLY intrared.r_role_group
ADD
  CONSTRAINT id_group_fk FOREIGN KEY (id_group) REFERENCES intrared.groups(id) ON DELETE CASCADE;
ALTER TABLE ONLY intrared.r_user_group
ADD
  CONSTRAINT id_group_fk FOREIGN KEY (id_group) REFERENCES intrared.groups(id) ON DELETE CASCADE;
ALTER TABLE ONLY intrared.r_role_group
ADD
  CONSTRAINT id_role_fk FOREIGN KEY (id_role) REFERENCES intrared.roles(id) ON DELETE CASCADE;
ALTER TABLE ONLY intrared.r_user_group
ADD
  CONSTRAINT id_user_fk FOREIGN KEY (id_user) REFERENCES intrared.users(id) ON DELETE CASCADE;