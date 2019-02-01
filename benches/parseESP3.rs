#[macro_use]
extern crate criterion;

use criterion::Criterion;

use enocean::enocean::*;

fn given_valid_a50401_enocean_message_then_return_corresponding_esp() {
    let received_message = vec![
        85, 0, 10, 7, 1, 235, 165, 0, 229, 204, 10, 5, 17, 114, 247, 0, 1, 255, 255, 255, 255, 54,
        0, 213,
    ];
    let esp3_packet: ESP3 = esp3_of_enocean_message(received_message).unwrap();
}
fn given_valid_f60201_enocean_message_then_return_corresponding_esp() {
    let received_message = vec![
        85, 0, 7, 7, 1, 122, 246, 0, 254, 245, 143, 212, 32, 2, 255, 255, 255, 255, 48, 0, 39,
    ];
    let esp3_packet: ESP3 = esp3_of_enocean_message(received_message).unwrap();
}

fn given_valid_f60202_enocean_message_then_return_corresponding_esp() {
    let received_message = vec![
        85, 0, 7, 7, 1, 122, 246, 0, 0, 49, 192, 249, 32, 2, 255, 255, 255, 255, 49, 0, 106,
    ];
    let esp3_packet: ESP3 = esp3_of_enocean_message(received_message).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parseESP3_a50401", |b| {
        b.iter(|| given_valid_a50401_enocean_message_then_return_corresponding_esp())
    });
    c.bench_function("parseESP3_f60201", |b| {
        b.iter(|| given_valid_a50401_enocean_message_then_return_corresponding_esp())
    });
    c.bench_function("parseESP3_f60202", |b| {
        b.iter(|| given_valid_a50401_enocean_message_then_return_corresponding_esp())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
