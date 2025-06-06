// src/game_actions.rs

use fleetcore::{BaseInputs, Command, FireInputs};
use methods::{FIRE_ELF, JOIN_ELF, REPORT_ELF, WAVE_ELF, WIN_ELF};
use risc0_zkvm::{default_prover, ExecutorEnv};

use crate::{send_receipt, unmarshal_data, unmarshal_fire, unmarshal_report, FormData};

pub async fn join_game(idata: FormData) -> String {
    let (gameid, fleetid, board, random) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    // Set up the zkVM environment
    let receipt = {
        let env = ExecutorEnv::builder()
            .write(&BaseInputs {
                gameid: gameid.clone(),
                fleet: fleetid.clone(),
                board: board.clone(),
                random: random.clone(),
            })
            .unwrap()
            .build()
            .unwrap();
        // Obtain the default prover
        let prover = default_prover();

        // Produce a receipt by proving the specified ELF binary
        prover.prove(env, JOIN_ELF).unwrap().receipt
    };

    // Send the receipt to the blockchain server
    send_receipt(Command::Join, receipt).await;

    "OK".to_string()
}

pub async fn fire(idata: FormData) -> String {
    // Unmarshal the input data
    let (gameid, fleetid, board, random, targetfleet, x, y) = match unmarshal_fire(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };
    // TO DO: Rebuild the receipt

    // Set up the zkVM environment
    let receipt = {
        let env = ExecutorEnv::builder()
            .write(&FireInputs {
                gameid: gameid.clone(),
                fleet: fleetid.clone(),
                board: board.clone(),
                random: random.clone(),
                target: targetfleet.clone(),
                pos: (x.clone() << 4) | y.clone(),
            })
            .unwrap()
            .build()
            .unwrap();
        // Obtain the default prover
        let prover = default_prover();

        // Produce a receipt by proving the specified ELF binary
        prover.prove(env, FIRE_ELF).unwrap().receipt
    };

    // Send the receipt to the blockchain server
    send_receipt(Command::Fire, receipt).await;

    // Return success message
    "OK".to_string()
}

pub async fn report(idata: FormData) -> String {
    let (gameid, fleetid, board, random, _report, x, y) = match unmarshal_report(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };
    // TO DO: Rebuild the receipt

    // Uncomment the following line when you are ready to send the receipt
    //send_receipt(Command::Fire, receipt).await
    // Comment out the following line when you are ready to send the receipt
    "OK".to_string()
}

pub async fn wave(idata: FormData) -> String {
    let (gameid, fleetid, board, random) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    // Set up the zkVM environment
    let receipt = {
        let env = ExecutorEnv::builder()
            .write(&BaseInputs {
                gameid: gameid.clone(),
                fleet: fleetid.clone(),
                board: board.clone(),
                random: random.clone(),
            })
            .unwrap()
            .build()
            .unwrap();
        // Obtain the default prover
        let prover = default_prover();

        // Produce a receipt by proving the specified ELF binary
        prover.prove(env, WAVE_ELF).unwrap().receipt
    };

    // Send the receipt to the blockchain server
    send_receipt(Command::Wave, receipt).await;

    "OK".to_string()
}

pub async fn win(idata: FormData) -> String {
    let (gameid, fleetid, board, random) = match unmarshal_data(&idata) {
        Ok(values) => values,
        Err(err) => return err,
    };

    // Set up the zkVM environment
    let receipt = {
        let env = ExecutorEnv::builder()
            .write(&BaseInputs {
                gameid: gameid.clone(),
                fleet: fleetid.clone(),
                board: board.clone(),
                random: random.clone(),
            })
            .unwrap()
            .build()
            .unwrap();
        // Obtain the default prover
        let prover = default_prover();

        // Produce a receipt by proving the specified ELF binary
        prover.prove(env, WIN_ELF).unwrap().receipt
    };

    // Send the receipt to the blockchain server
    send_receipt(Command::Win, receipt).await;

    "OK".to_string()
}
