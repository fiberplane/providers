use super::{ClientCommon, ClientError};
use crate::{api::paginate::paginate_vec, config::Config, types::api::cloudwatch_logs::*};
use fiberplane_pdk::prelude::{Error, Timestamp};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) common: ClientCommon,
}

impl Client {
    fn from_config(config: &Config) -> Self {
        Self {
            common: ClientCommon {
                service: "logs".to_string(),
                host: format!("logs.{}.amazonaws.com", config.region),
                endpoint: format!("https://logs.{}.amazonaws.com", config.region),
                region: config.region.clone(),
                access_key_id: config.access_key_id.clone(),
                secret_access_key: config.secret_access_key.clone(),
            },
        }
    }

    pub async fn list_log_groups(
        &self,
        log_group_name_pattern: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<LogGroup>, Error> {
        let init_request = DescribeLogGroupsRequest {
            account_identifiers: None,
            include_linked_accounts: None,
            limit: None,
            log_group_name_pattern,
            log_group_name_prefix: None,
            next_token: None,
        };
        return paginate_vec(
            &self.common,
            init_request,
            |log_groups_payload: DescribeLogGroupsResponse| {
                Some(log_groups_payload.log_groups.into_iter())
            },
            |log_groups_payload| log_groups_payload.next_token.clone(),
            limit,
        )
        .await;
    }

    pub async fn describe_queries(
        &self,
        log_group_name: Option<String>,
        status: Option<QueryStatus>,
        limit: Option<usize>,
    ) -> Result<Vec<QueryInfo>, Error> {
        let init_request = DescribeQueriesRequest {
            next_token: None,
            log_group_name,
            max_results: limit.and_then(|l| l.try_into().ok()),
            status,
        };
        return paginate_vec(
            &self.common,
            init_request,
            |queries_payload: DescribeQueriesResponse| Some(queries_payload.queries.into_iter()),
            |queries_payload: &DescribeQueriesResponse| queries_payload.next_token.clone(),
            limit,
        )
        .await;
    }

    pub async fn start_query(
        &self,
        query: String,
        start_time: Timestamp,
        end_time: Timestamp,
        log_group_names: Vec<String>,
        limit: Option<usize>,
    ) -> Result<String, Error> {
        let request = StartQueryRequest {
            limit: limit.map(|l| l.try_into().unwrap()),
            log_group_names: Some(log_group_names),
            log_group_identifiers: None,
            log_group_name: None,
            query_string: query,
            start_time: start_time.into(),
            end_time: end_time.into(),
        };
        let start_query_payload: StartQueryResponse = self.common.send(request, None).await?;
        Ok(start_query_payload.query_id)
    }

    pub async fn get_query_results(
        &self,
        query_id: String,
    ) -> Result<GetQueryResultsResponse, ClientError> {
        self.common
            .send(GetQueryResultsRequest { query_id }, None)
            .await
    }

    pub async fn get_log_details(
        &self,
        log_record_pointer: String,
        unmask: Option<bool>,
    ) -> Result<HashMap<String, String>, Error> {
        let request = GetLogRecordRequest {
            log_record_pointer,
            unmask,
        };
        self.common
            .send(request, None)
            .await
            .map_err(Into::into)
            .map(|r: GetLogRecordResponse| r.log_record)
    }
}

impl From<&Config> for Client {
    fn from(conf: &Config) -> Self {
        Self::from_config(conf)
    }
}
