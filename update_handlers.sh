#!/bin/bash

# Update observation handlers
sed -i '' 's/pub async fn create_observation(/pub async fn create_observation(\n    auth: OptionalAuthUser,/g' src/api/handlers/observation.rs
sed -i '' 's/pub async fn get_observation(/pub async fn get_observation(\n    auth: OptionalAuthUser,/g' src/api/handlers/observation.rs
sed -i '' 's/pub async fn update_observation(/pub async fn update_observation(\n    auth: OptionalAuthUser,/g' src/api/handlers/observation.rs
sed -i '' 's/pub async fn delete_observation(/pub async fn delete_observation(\n    auth: OptionalAuthUser,/g' src/api/handlers/observation.rs
sed -i '' 's/pub async fn search_observations(/pub async fn search_observations(\n    auth: OptionalAuthUser,/g' src/api/handlers/observation.rs

# Update condition handlers
sed -i '' 's/pub async fn create_condition(/pub async fn create_condition(\n    auth: OptionalAuthUser,/g' src/api/handlers/condition.rs
sed -i '' 's/pub async fn get_condition(/pub async fn get_condition(\n    auth: OptionalAuthUser,/g' src/api/handlers/condition.rs
sed -i '' 's/pub async fn update_condition(/pub async fn update_condition(\n    auth: OptionalAuthUser,/g' src/api/handlers/condition.rs
sed -i '' 's/pub async fn delete_condition(/pub async fn delete_condition(\n    auth: OptionalAuthUser,/g' src/api/handlers/condition.rs
sed -i '' 's/pub async fn search_conditions(/pub async fn search_conditions(\n    auth: OptionalAuthUser,/g' src/api/handlers/condition.rs

# Update encounter handlers
sed -i '' 's/pub async fn create_encounter(/pub async fn create_encounter(\n    auth: OptionalAuthUser,/g' src/api/handlers/encounter.rs
sed -i '' 's/pub async fn get_encounter(/pub async fn get_encounter(\n    auth: OptionalAuthUser,/g' src/api/handlers/encounter.rs
sed-i '' 's/pub async fn update_encounter(/pub async fn update_encounter(\n    auth: OptionalAuthUser,/g' src/api/handlers/encounter.rs
sed -i '' 's/pub async fn delete_encounter(/pub async fn delete_encounter(\n    auth: OptionalAuthUser,/g' src/api/handlers/encounter.rs
sed -i '' 's/pub async fn search_encounters(/pub async fn search_encounters(\n    auth: OptionalAuthUser,/g' src/api/handlers/encounter.rs

echo "Done"
