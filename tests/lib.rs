use scrypto_test::prelude::*;
use scrypto::prelude::Url;

use locker::locker_mod_test::*;
use locker::{LockContents, LockReceipt};

#[test]
fn test_lock_fungible() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = 
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;

    let owner_resource = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(1), &mut env)?
        .resource_address(&mut env)?;
    let mut locker = Locker::new(owner_resource, Url::of("https://example.com"), Url::of("https://example.com/icon.png"), package_address, &mut env)?;
    let token = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(100), &mut env)?;
    let resource = token.resource_address(&mut env)?;
    let amount = token.amount(&mut env)?;

    let mut current_time = env.get_current_time();

    // Act
    let receipt = locker.lock(
        token, current_time.add_seconds(1),
        "Test Lock".to_string(),
        "Test Lock Description".to_string(),
        Url::of("https://example.com/key.png"),
        &mut env
    )?;
    let receipt_resource = receipt.resource_address(&mut env)?;
    let receipt_id = receipt.non_fungible_local_ids(&mut env)?[0].clone();

    // Assert
    let receipt_data: LockReceipt = ResourceManager(receipt_resource).get_non_fungible_data(receipt_id.clone(), &mut env)?;
    assert_eq!(receipt_data.name, "Test Lock");
    assert_eq!(receipt_data.description, "Test Lock Description");
    assert_eq!(receipt_data.key_image_url, Url::of("https://example.com/key.png"));
    assert_eq!(receipt_data.resource, resource);
    assert_eq!(receipt_data.locked_contents, LockContents::Fungible(amount));
    assert_eq!(receipt_data.locked_at, current_time);
    assert_eq!(receipt_data.unlockable_at, current_time.add_seconds(1));
    assert_eq!(receipt_data.unlocked_at, None);

    // Act
    current_time = current_time.add_seconds(2).unwrap();
    env.set_current_time(current_time);
    let unlocks = locker.unlock(receipt, &mut env)?;

    // Assert
    assert_eq!(unlocks[0].resource_address(&mut env)?, resource);
    assert_eq!(unlocks[0].amount(&mut env)?, amount);

    let receipt_data: LockReceipt = ResourceManager(receipt_resource).get_non_fungible_data(receipt_id, &mut env)?;
    assert_eq!(receipt_data.unlocked_at, Some(current_time));

    Ok(())
}

#[test]
fn test_lock_non_fungible() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = 
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;

    let owner_resource = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(1), &mut env)?
        .resource_address(&mut env)?;
    let mut locker = Locker::new(owner_resource, Url::of("https://example.com"), Url::of("https://example.com/icon.png"), package_address, &mut env)?;
    let nft = ResourceBuilder::new_ruid_non_fungible(OwnerRole::None)
        .mint_initial_supply(vec![(), (), ()], &mut env)?;
    let resource = nft.resource_address(&mut env)?;
    let nft_ids = nft.non_fungible_local_ids(&mut env)?;

    let mut current_time = env.get_current_time();

    // Act
    let receipt = locker.lock(
        nft, current_time.add_seconds(1),
        "Test Lock".to_string(),
        "Test Lock Description".to_string(),
        Url::of("https://example.com/key.png"),
        &mut env
    )?;
    let receipt_resource = receipt.resource_address(&mut env)?;
    let receipt_id = receipt.non_fungible_local_ids(&mut env)?[0].clone();

    // Assert
    let receipt_data: LockReceipt = ResourceManager(receipt_resource).get_non_fungible_data(receipt_id.clone(), &mut env)?;
    assert_eq!(receipt_data.name, "Test Lock");
    assert_eq!(receipt_data.description, "Test Lock Description");
    assert_eq!(receipt_data.key_image_url, Url::of("https://example.com/key.png"));
    assert_eq!(receipt_data.resource, resource);
    assert_eq!(receipt_data.locked_contents, LockContents::NonFungible(nft_ids.clone()));
    assert_eq!(receipt_data.locked_at, current_time);
    assert_eq!(receipt_data.unlockable_at, current_time.add_seconds(1));
    assert_eq!(receipt_data.unlocked_at, None);

    // Act
    current_time = current_time.add_seconds(2).unwrap();
    env.set_current_time(current_time);
    let unlocks = locker.unlock(receipt, &mut env)?;

    // Assert
    assert_eq!(unlocks[0].resource_address(&mut env)?, resource);
    assert_eq!(unlocks[0].non_fungible_local_ids(&mut env)?, nft_ids);

    let receipt_data: LockReceipt = ResourceManager(receipt_resource).get_non_fungible_data(receipt_id, &mut env)?;
    assert_eq!(receipt_data.unlocked_at, Some(current_time));

    Ok(())
}

#[test]
fn test_lock_unlock_multiple() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = 
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;

    let owner_resource = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(1), &mut env)?
        .resource_address(&mut env)?;
    let mut locker = Locker::new(owner_resource, Url::of("https://example.com"), Url::of("https://example.com/icon.png"), package_address, &mut env)?;
    let token_1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(100), &mut env)?;
    let resource_1 = token_1.resource_address(&mut env)?;
    let amount_1 = token_1.amount(&mut env)?;

    let token_2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(100), &mut env)?;
    let resource_2 = token_2.resource_address(&mut env)?;
    let amount_2 = token_2.amount(&mut env)?;

    let mut current_time = env.get_current_time();

    let receipts = locker.lock(
        token_1, current_time.add_seconds(1),
        "Test Lock".to_string(),
        "Test Lock Description".to_string(),
        Url::of("https://example.com/key.png"),
        &mut env
    )?;
    let receipt_2 = locker.lock(
        token_2, current_time.add_seconds(1),
        "Test Lock".to_string(),
        "Test Lock Description".to_string(),
        Url::of("https://example.com/key.png"),
        &mut env
    )?;
    receipts.put(receipt_2, &mut env)?;

    // Act
    current_time = current_time.add_seconds(2).unwrap();
    env.set_current_time(current_time);
    let unlocks = locker.unlock(receipts, &mut env)?;

    // Assert
    assert_eq!(unlocks[0].resource_address(&mut env)?, resource_1);
    assert_eq!(unlocks[0].amount(&mut env)?, amount_1);
    assert_eq!(unlocks[1].resource_address(&mut env)?, resource_2);
    assert_eq!(unlocks[1].amount(&mut env)?, amount_2);

    Ok(())
}

#[test]
fn test_lock_unlock_before_never() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = 
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;

    let owner_resource = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(1), &mut env)?
        .resource_address(&mut env)?;
    let mut locker = Locker::new(owner_resource, Url::of("https://example.com"), Url::of("https://example.com/icon.png"), package_address, &mut env)?;
    let token = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(100), &mut env)?;

    let receipt = locker.lock(
        token, 
        None,
        "Test Lock".to_string(),
        "Test Lock Description".to_string(),
        Url::of("https://example.com/key.png"),
        &mut env
    )?;

    // Act
    let result = locker.unlock(receipt, &mut env);

    // Assert
    match result.err() {
        Some(RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg))) => {
            assert!(msg.contains(format!("Item can not yet be unlocked, unlockable at: {:?}", None::<Instant>).as_str()), "{}", msg);
        },
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_lock_unlock_before_some() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = 
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;

    let owner_resource = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(1), &mut env)?
        .resource_address(&mut env)?;
    let mut locker = Locker::new(owner_resource, Url::of("https://example.com"), Url::of("https://example.com/icon.png"), package_address, &mut env)?;
    let token = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(100), &mut env)?;

    let current_time = env.get_current_time();

    let receipt = locker.lock(
        token, 
        current_time.add_seconds(1),
        "Test Lock".to_string(),
        "Test Lock Description".to_string(),
        Url::of("https://example.com/key.png"),
        &mut env
    )?;

    // Act
    let result = locker.unlock(receipt, &mut env);

    // Assert
    match result.err() {
        Some(RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg))) => {
            assert!(msg.contains(format!("Item can not yet be unlocked, unlockable at: {:?}", current_time.add_seconds(1)).as_str()), "{}", msg);
        },
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_lock_unlock_wrong_receipt() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = 
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;

    let owner_resource = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(1), &mut env)?
        .resource_address(&mut env)?;
    let mut locker_1 = Locker::new(owner_resource, Url::of("https://example.com"), Url::of("https://example.com/icon.png"), package_address, &mut env)?;
    let token_1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(100), &mut env)?;
    let mut locker_2 = Locker::new(owner_resource, Url::of("https://example.com"), Url::of("https://example.com/icon.png"), package_address, &mut env)?;
    let token_2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(dec!(100), &mut env)?;

    let mut current_time = env.get_current_time();

    let _receipt = locker_1.lock(
        token_1, 
        current_time.add_seconds(1),
        "Test Lock".to_string(),
        "Test Lock Description".to_string(),
        Url::of("https://example.com/key.png"),
        &mut env
    )?;
    let fake_receipt = locker_2.lock(
        token_2, 
        current_time.add_seconds(1),
        "Test Lock".to_string(),
        "Test Lock Description".to_string(),
        Url::of("https://example.com/key.png"),
        &mut env
    )?;

    // Act
    current_time = current_time.add_seconds(2).unwrap();
    env.set_current_time(current_time);
    let result = locker_1.unlock(fake_receipt, &mut env);

    // Assert
    match result.err() {
        Some(RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg))) => {
            assert!(msg.contains("Invalid lock receipts"), "{}", msg);
        },
        _ => panic!(),
    }

    Ok(())
}
