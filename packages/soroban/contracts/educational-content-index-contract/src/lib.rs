#![no_std]

mod error;
mod events;
mod metadata;
mod search;
mod storage;
mod validate;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, symbol_short, Env, String, Symbol, Vec};

use crate::error::Error;
use crate::events::Events;
use crate::metadata::Content;
use crate::search::search_content;
use crate::storage::ContentStorage;

const INITIALIZED_KEY: Symbol = symbol_short!("INIT");

#[contract]
pub struct ContentSearchContract;

#[contractimpl]
impl ContentSearchContract {
    pub fn initialize(env: Env) {
        let storage = env.storage().instance();

        // Verificar si ya está inicializado
        if storage.has(&INITIALIZED_KEY) {
            panic!("Contract already initialized");
        }

        // Inicializar el almacenamiento
        ContentStorage::initialize(&env);

        // Marcar como inicializado
        storage.set(&INITIALIZED_KEY, &true);
        storage.extend_ttl(50, 100);
    }

    pub fn search_content(env: Env, subject: String) -> Result<Vec<Content>, Error> {
        // Verificar que el contrato está inicializado
        if !env.storage().instance().has(&INITIALIZED_KEY) {
            return Err(Error::NotInitialized);
        }

        // Validar el subject
        if !crate::validate::validate_subject(&subject) {
            return Err(Error::InvalidInput);
        }

        let results = search_content(&env, subject.clone())?;

        // Emit search performed event
        Events::search_performed(&env, &subject, results.len() as u32);

        Ok(results)
    }

    pub fn add_content(
        env: Env,
        title: String,
        description: String,
        subject_tags: Vec<String>,
        content_url: String,
        author: Option<String>,
        difficulty_level: Option<String>,
        creation_date: Option<u64>,
    ) -> Result<u64, Error> {
        // Verificar que el contrato está inicializado
        if !env.storage().instance().has(&INITIALIZED_KEY) {
            return Err(Error::NotInitialized);
        }

        // Obtener y actualizar el ID
        let storage = env.storage().instance();
        let id = storage
            .get::<Symbol, u64>(&symbol_short!("NEXT_ID"))
            .unwrap_or(0)
            + 1;
        storage.set(&symbol_short!("NEXT_ID"), &id);

        // Crear el contenido
        let content = Content {
            id,
            title,
            description,
            subject_tags,
            content_url,
            author,
            difficulty_level,
            creation_date,
        };

        // Validar el contenido
        crate::validate::validate_content(&env, &content)?;

        // Guardar el contenido
        ContentStorage::set_content(&env, &content);

        // Emit content added event
        Events::content_added(&env, &content);

        Ok(id)
    }

    pub fn update_content(
        env: Env,
        content_id: u64,
        title: String,
        description: String,
        subject_tags: Vec<String>,
        content_url: String,
        author: Option<String>,
        difficulty_level: Option<String>,
        creation_date: Option<u64>,
    ) -> Result<(), Error> {
        // Verify contract is initialized
        if !env.storage().instance().has(&INITIALIZED_KEY) {
            return Err(Error::NotInitialized);
        }

        // Get existing content
        let existing_content =
            ContentStorage::get_content_by_id(&env, content_id).ok_or(Error::ContentNotFound)?;

        // Create updated content
        let updated_content = Content {
            id: content_id,
            title,
            description,
            subject_tags,
            content_url,
            author,
            difficulty_level,
            creation_date,
        };

        // Validate the content
        crate::validate::validate_content(&env, &updated_content)?;

        // Save the content
        ContentStorage::set_content(&env, &updated_content);

        // Emit content updated event
        Events::content_updated(&env, &existing_content, &updated_content);

        Ok(())
    }

    /// Search content using multiple tags (OR operation) with indexed search for better performance
    pub fn search_content_multi_tag(env: Env, tags: Vec<String>) -> Result<Vec<Content>, Error> {
        // Verify contract is initialized
        if !env.storage().instance().has(&INITIALIZED_KEY) {
            return Err(Error::NotInitialized);
        }

        // Validate tags
        for tag in tags.iter() {
            if !crate::validate::validate_subject(&tag) {
                return Err(Error::InvalidInput);
            }
        }

        let results = crate::search::engine::search_content_multi_tag(&env, tags)?;

        // Emit search performed event
        Events::search_performed(
            &env,
            &String::from_str(&env, "multi-tag"),
            results.len() as u32,
        );

        Ok(results)
    }

    /// Get content by ID using indexed lookup for better performance
    pub fn get_content_by_id(env: Env, content_id: u64) -> Option<Content> {
        // Verify contract is initialized
        if !env.storage().instance().has(&INITIALIZED_KEY) {
            return None;
        }

        // Try indexed lookup first, fallback to linear search for compatibility
        if let Some(content) = ContentStorage::get_content_by_id_indexed(&env, content_id) {
            Some(content)
        } else {
            ContentStorage::get_content_by_id(&env, content_id)
        }
    }

    /// Rebuild search indices - useful for migrating existing content to indexed search
    /// This is an administrative function that should be called after contract upgrades
    pub fn rebuild_search_indices(env: Env) -> Result<(), Error> {
        // Verify contract is initialized
        if !env.storage().instance().has(&INITIALIZED_KEY) {
            return Err(Error::NotInitialized);
        }

        ContentStorage::rebuild_indices(&env);

        Ok(())
    }
}
