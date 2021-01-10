use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;
/// 创建存证
#[test]
fn create_claim_works(){
    new_test_ext().execute_with(||{
        let claim = vec![0, 1];
        assert_ok!(PoeModule::claim_created(Origin::signed(1), claim.clone())); // 断言运行结果
        assert_eq!(Proofs::<Test>::get(&claim), (1, frame_system::Module::<Test>::block_number()));
    })
}
/// 创建的存证已存在
#[test]
fn create_claim_failed_when_claim_already_exist(){
    new_test_ext().execute_with(||{

        let claim = vec![0, 1];
        let _ = PoeModule::claim_created(Origin::signed(1), claim.clone());
        assert_noop!( // 断言运行结果为ProofAlreadyClaimed
            PoeModule::claim_created(Origin::signed(1), claim.clone()),
            Error::<Test>::ProofAlreadyClaimed
        );
    })
}

/// 撤销存证
#[test]
fn revoke_claim_works(){
    new_test_ext().execute_with(||{
        let claim = vec![0, 1];
        let _ = PoeModule::claim_created(Origin::signed(1), claim.clone());
        assert_ok!(PoeModule::claim_revoked(Origin::signed(1), claim.clone())); // 断言运行结果
    })
}
/// 撤销的存证不存在
#[test]
fn revoke_claim_failed_when_claim_is_not_exist(){
    new_test_ext().execute_with(||{
        let claim = vec![0, 1];
        assert_noop!( // 断言运行结果为NoSuchProof
            PoeModule::claim_revoked(Origin::signed(1), claim.clone()),
            Error::<Test>::NoSuchProof
        );
    })
}
/// 不是该存证拥有者
#[test]
fn revoke_claim_failed_when_is_not_claim_owner(){
    new_test_ext().execute_with(||{
        let claim = vec![0, 1];
        let _ = PoeModule::claim_created(Origin::signed(1), claim.clone());
        assert_noop!( // 断言运行结果为NotProofOwner
            PoeModule::claim_revoked(Origin::signed(2), claim.clone()),
            Error::<Test>::NotProofOwner
        );
    })
}
/// 转移存证
#[test]
fn transfer_claim_works(){
    new_test_ext().execute_with(||{
        let claim = vec![0, 1];
        assert_noop!( // 不存在
            PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2),
            Error::<Test>::NoSuchProof
        );
        let _ = PoeModule::claim_created(Origin::signed(1), claim.clone());
        assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2)); // 断言运行结果，成功
        assert_noop!( // 非拥有者
            PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2),
            Error::<Test>::NotProofOwner
        );
    })
}

/// 存证长度限制
#[test]
fn create_claim_failed_when_it_is_too_long(){
    new_test_ext().execute_with(||{
        let claim1 = vec![0, 1, 2, 3, 4, 5, 6 ,7 ,8 ,9];
        assert_ok!(PoeModule::claim_created(Origin::signed(1), claim1.clone())); // 断言运行结果
        let claim2 = vec![0, 1, 2, 3, 4, 5, 6 ,7 ,8 ,9, 10];
        assert_noop!(
                PoeModule::claim_created(Origin::signed(1), claim2.clone()),
                Error::<Test>::ProofTooLong
            );
    })
}