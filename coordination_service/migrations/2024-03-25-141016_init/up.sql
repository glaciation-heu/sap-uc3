CREATE TABLE csconfig (
  id serial NOT NULL,
  prime VARCHAR(255) NOT NULL,
  r VARCHAR(255) NOT NULL,
  rinv VARCHAR(255) NOT NULL,
  no_ssl_validation BOOL NOT NULL,
  CONSTRAINT csconfig_pkey PRIMARY KEY (id)
);

CREATE TABLE csprovider (
  config_id INT NOT NULL,
  id INT NOT NULL,
  amphora_service_url VARCHAR(255) NOT NULL,
  castor_service_url VARCHAR(255) NOT NULL,
  ephemeral_service_url VARCHAR(255) NOT NULL,
  base_url VARCHAR(255) NOT NULL,
  CONSTRAINT csprovider_pkey PRIMARY KEY(config_id, id),
  CONSTRAINT fk_provider_config
    FOREIGN KEY(config_id)
      REFERENCES csconfig(id)
      ON DELETE CASCADE
);

CREATE TABLE collaborations (
  id serial NOT NULL,
  name VARCHAR(255) NOT NULL,
  mpc_program text NOT NULL,
  csv_specification text NOT NULL,
  participation_number INT NOT NULL DEFAULT 0,
  config_id INT NOT NULL,
  output_parties text [],
  CONSTRAINT collab_pkey PRIMARY KEY (id),
  CONSTRAINT collab_config_fkey
    FOREIGN KEY(config_id)
      REFERENCES csconfig(id)
      ON DELETE CASCADE
);

CREATE TABLE participations (
  collaboration_id INT NOT NULL,
  party_id INT NOT NULL,
  secret_ids CHAR(36) [],
  CONSTRAINT participation_pkey PRIMARY KEY (collaboration_id, party_id),
  CONSTRAINT fk_collab_participation
    FOREIGN KEY(collaboration_id)
      REFERENCES collaborations(id)
      ON DELETE CASCADE
);