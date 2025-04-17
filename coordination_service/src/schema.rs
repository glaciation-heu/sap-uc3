// @generated automatically by Diesel CLI.

diesel::table! {
    collaborations (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        mpc_program -> Text,
        csv_specification -> Text,
        participation_number -> Int4,
        config_id -> Int4,
        output_parties -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    computation_results (collab_id) {
        collab_id -> Int4,
        result_ids -> Nullable<Array<Nullable<Text>>>,
        finished -> Bool,
        #[max_length = 255]
        error -> Nullable<Varchar>,
    }
}

diesel::table! {
    csconfig (id) {
        id -> Int4,
        #[max_length = 255]
        prime -> Varchar,
        #[max_length = 255]
        r -> Varchar,
        #[max_length = 255]
        rinv -> Varchar,
        no_ssl_validation -> Bool,
    }
}

diesel::table! {
    csprovider (config_id, id) {
        config_id -> Int4,
        id -> Int4,
        #[max_length = 255]
        amphora_service_url -> Varchar,
        #[max_length = 255]
        castor_service_url -> Varchar,
        #[max_length = 255]
        ephemeral_service_url -> Varchar,
        #[max_length = 255]
        base_url -> Varchar,
    }
}

diesel::table! {
    participations (collaboration_id, party_id) {
        collaboration_id -> Int4,
        party_id -> Int4,
        secret_ids -> Nullable<Array<Nullable<Bpchar>>>,
    }
}

diesel::joinable!(collaborations -> csconfig (config_id));
diesel::joinable!(computation_results -> collaborations (collab_id));
diesel::joinable!(csprovider -> csconfig (config_id));
diesel::joinable!(participations -> collaborations (collaboration_id));

diesel::allow_tables_to_appear_in_same_query!(
    collaborations,
    computation_results,
    csconfig,
    csprovider,
    participations,
);
