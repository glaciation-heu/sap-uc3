CREATE TABLE computation_results (
  collab_id INT NOT NULL,
  result_ids text [],
  finished BOOLEAN NOT NULL,
  error VARCHAR(255),
  CONSTRAINT results_pkey PRIMARY KEY (collab_id),
  CONSTRAINT fk_collab_results
    FOREIGN KEY(collab_id)
      REFERENCES collaborations(id)
      ON DELETE CASCADE
)