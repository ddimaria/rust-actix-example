use crate::auth::hash;
use crate::database::PoolType;
use crate::errors::ApiError;
use crate::handlers::user::{UserResponse, UsersResponse};
use crate::schema::users;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Identifiable, Insertable)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub created_by: String,
    pub created_at: NaiveDateTime,
    pub updated_by: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewUser {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub created_by: String,
    pub updated_by: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, AsChangeset)]
#[table_name = "users"]
pub struct UpdateUser {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub updated_by: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
}

/// Get all users
pub fn get_all(pool: &PoolType) -> Result<UsersResponse, ApiError> {
    use crate::schema::users::dsl::users;

    let conn = pool.get()?;
    let all_users = users.load(&conn)?;

    Ok(all_users.into())
}

/// Find a user by the user's id or error out
pub fn find(pool: &PoolType, user_id: Uuid) -> Result<UserResponse, ApiError> {
    use crate::schema::users::dsl::{id, users};

    let not_found = format!("User {} not found", user_id);
    let conn = pool.get()?;
    let user = users
        .filter(id.eq(user_id.to_string()))
        .first::<User>(&conn)
        .map_err(|_| ApiError::NotFound(not_found))?;

    Ok(user.into())
}

/// Find a user by the user's authentication information (email + password)
/// Return an Unauthorized error if it doesn't match
pub fn find_by_auth(
    pool: &PoolType,
    user_email: &str,
    user_password: &str,
) -> Result<UserResponse, ApiError> {
    use crate::schema::users::dsl::{email, password, users};

    let conn = pool.get()?;
    let user = users
        .filter(email.eq(user_email.to_string()))
        .filter(password.eq(user_password.to_string()))
        .first::<User>(&conn)
        .map_err(|_| ApiError::Unauthorized("Invalid login".into()))?;
    Ok(user.into())
}

/// Create a new user
pub fn create(pool: &PoolType, new_user: &User) -> Result<UserResponse, ApiError> {
    use crate::schema::users::dsl::users;

    let conn = pool.get()?;
    diesel::insert_into(users).values(new_user).execute(&conn)?;
    Ok(new_user.clone().into())
}

/// Update a user
pub fn update(pool: &PoolType, update_user: &UpdateUser) -> Result<UserResponse, ApiError> {
    use crate::schema::users::dsl::{id, users};

    let conn = pool.get()?;
    diesel::update(users)
        .filter(id.eq(update_user.id.clone()))
        .set(update_user)
        .execute(&conn)?;
    find(&pool, Uuid::parse_str(&update_user.id)?)
}

/// Delete a user
pub fn delete(pool: &PoolType, user_id: Uuid) -> Result<(), ApiError> {
    use crate::schema::users::dsl::{id, users};

    let conn = pool.get()?;
    diesel::delete(users)
        .filter(id.eq(user_id.to_string()))
        .execute(&conn)?;
    Ok(())
}

impl From<NewUser> for User {
    fn from(user: NewUser) -> Self {
        User {
            id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            password: hash(&user.password),
            created_by: user.created_by,
            created_at: Utc::now().naive_utc(),
            updated_by: user.updated_by,
            updated_at: Utc::now().naive_utc(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tests::helpers::tests::get_pool;

    pub fn get_all_users() -> Result<UsersResponse, ApiError> {
        let pool = get_pool();
        get_all(&pool)
    }

    pub fn create_user() -> Result<UserResponse, ApiError> {
        let user_id = Uuid::new_v4();
        let new_user = NewUser {
            id: user_id.to_string(),
            first_name: "Model".to_string(),
            last_name: "Test".to_string(),
            email: "model-test@nothing.org".to_string(),
            password: "123456".to_string(),
            created_by: user_id.to_string(),
            updated_by: user_id.to_string(),
        };
        let user: User = new_user.into();
        create(&get_pool(), &user)
    }

    #[test]
    fn it_gets_a_user() {
        let users = get_all_users();
        assert!(users.is_ok());
    }

    #[test]
    fn test_find() {
        let users = get_all_users().unwrap();
        let user = &users.0[0];
        let found_user = find(&get_pool(), user.id).unwrap();
        assert_eq!(user, &found_user);
    }

    #[test]
    fn it_doesnt_find_a_user() {
        let user_id = Uuid::new_v4();
        let not_found_user = find(&get_pool(), user_id);
        assert!(not_found_user.is_err());
    }

    #[test]
    fn it_creates_a_user() {
        let created = create_user();
        assert!(created.is_ok());
        let unwrapped = created.unwrap();
        let found_user = find(&get_pool(), unwrapped.id.clone()).unwrap();
        assert_eq!(unwrapped, found_user);
    }

    #[test]
    fn it_updates_a_user() {
        let users = get_all_users().unwrap();
        let user = &users.0[1];
        let update_user = UpdateUser {
            id: user.id.to_string(),
            first_name: "ModelUpdate".to_string(),
            last_name: "TestUpdate".to_string(),
            email: "model-update-test@nothing.org".to_string(),
            updated_by: user.id.to_string(),
        };
        let updated = update(&get_pool(), &update_user);
        assert!(updated.is_ok());
        let found_user = find(&get_pool(), user.id).unwrap();
        assert_eq!(updated.unwrap(), found_user);
    }

    #[test]
    fn it_fails_to_update_a_nonexistent_user() {
        let user_id = Uuid::new_v4();
        let update_user = UpdateUser {
            id: user_id.to_string(),
            first_name: "ModelUpdateFailure".to_string(),
            last_name: "TestUpdateFailure".to_string(),
            email: "model-update-failure-test@nothing.org".to_string(),
            updated_by: user_id.to_string(),
        };
        let updated = update(&get_pool(), &update_user);
        assert!(updated.is_err());
    }

    #[test]
    fn it_deletes_a_user() {
        let created = create_user();
        let user_id = created.unwrap().id;
        let user = find(&get_pool(), user_id);
        assert!(user.is_ok());
        delete(&get_pool(), user_id).unwrap();
        let user = find(&get_pool(), user_id);
        assert!(user.is_err());
    }
}
