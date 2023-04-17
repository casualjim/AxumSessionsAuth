use crate::{AuthCache, AuthConfig, AuthSessionService, Authentication};
use axum_session::DatabasePool;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt, hash::Hash, marker::PhantomData};
use time::{ext::NumericalDuration, OffsetDateTime};
use tower_layer::Layer;

/// Layer used to generate an AuthSessionService.
///
#[derive(Clone, Debug)]
pub struct AuthSessionLayer<User, Type, Sess, Pool>
where
    Type: Eq + Default + Clone + Send + Sync + Hash + Serialize + DeserializeOwned + 'static,
{
    pub(crate) pool: Option<Pool>,
    pub(crate) config: AuthConfig<Type>,
    pub phantom_user: PhantomData<User>,
    pub phantom_session: PhantomData<Sess>,
    pub phantom_type: PhantomData<Type>,
}

impl<User, Type, Sess, Pool> AuthSessionLayer<User, Type, Sess, Pool>
where
    User: Authentication<User, Type, Pool> + Clone + Send,
    Pool: Clone + Send + Sync + fmt::Debug + 'static,
    Type: Eq + Default + Clone + Send + Sync + Hash + Serialize + DeserializeOwned + 'static,
    Sess: DatabasePool + Clone + Sync + Send + 'static,
{
    /// Used to generate an AuthSessionLayer with will call Towers layer() to generate a AuthSessionService.
    ///
    /// contains an Optional axum_session_database Pool for Sqlx database lookups against Right tokens.
    ///
    /// # Examples
    /// ```rust no_run
    ///    let layer = AuthSessionLayer::<User, i64, Sess, Pool>::new(None);
    /// ```
    ///
    pub fn new(pool: Option<Pool>) -> Self {
        Self {
            pool,
            config: AuthConfig::default(),
            phantom_user: PhantomData::default(),
            phantom_session: PhantomData::default(),
            phantom_type: PhantomData::default(),
        }
    }

    #[must_use]
    pub fn with_config(mut self, config: AuthConfig<Type>) -> Self {
        self.config = config;
        self
    }
}

impl<S, User, Type, Sess, Pool> Layer<S> for AuthSessionLayer<User, Type, Sess, Pool>
where
    User: Authentication<User, Type, Pool> + Clone + Send,
    Pool: Clone + Send + Sync + fmt::Debug + 'static,
    Type: Eq + Default + Clone + Send + Sync + Hash + Serialize + DeserializeOwned + 'static,
    Sess: DatabasePool + Clone + fmt::Debug + Sync + Send + 'static,
{
    type Service = AuthSessionService<S, User, Type, Sess, Pool>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthSessionService {
            pool: self.pool.clone(),
            config: self.config.clone(),
            cache: AuthCache::<User, Type, Pool>::new(OffsetDateTime::now_utc() + 1.hours()),
            inner,
            phantom_session: PhantomData::default(),
        }
    }
}
