use std::marker::PhantomData;
use ff::{Field, PrimeField, PrimeFieldBits};
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value, SimpleFloorPlanner},
    plonk::{Advice, Instance, Assigned, Column, ConstraintSystem,
        Constraints, Error, Expression, Selector, Circuit},
    poly::Rotation,
    pasta::Fp,
    dev::{FailureLocation, MockProver, VerifyFailure},    
};

#[derive(Debug, Clone)]
struct CardConfig {
    qty: Column<Advice>,
    suite: Column<Advice>,
    rank: Column<Advice>,
    pub_qty: Column<Instance>,
    pub_suite: Column<Instance>,
    pub_rank: Column<Instance>,
    s_check: Selector,
}

#[derive(Debug, Clone)]
struct CardChip<F:PrimeField> {
    config: CardConfig,
    _marker: PhantomData<F>,
}

#[derive(Debug, Clone)]
struct CardConstrained<F: PrimeField>
    (AssignedCell<Assigned<F>, F>);

impl<F: PrimeField> CardChip<F> {

    fn construct(config: CardConfig) -> Self {
        Self { config, _marker: PhantomData}
    }

    fn configure(meta: &mut ConstraintSystem<F>, qty: Column<Advice>,
                suite: Column<Advice>, rank: Column<Advice>,
                pub_qty: Column<Instance>, pub_suite: Column<Instance>,
                pub_rank: Column<Instance>, s_check: Selector) -> CardConfig {

        meta.enable_equality(pub_qty);
        meta.enable_equality(pub_suite);
        meta.enable_equality(pub_rank);

        let qty_check = |value: Expression<F>| {
            // Quantity check 1 or 2 or 3 or 4
            (1..4).fold(value.clone(), |acc, i| {
                acc * (Expression::Constant(F::from(i as u64)) - value.clone())
            })
        };

        let suite_check = |value: Expression<F>| {
            // Suite check Hearts or Spades or Diamonds or Flowers
            (1..4).fold(value.clone(), |acc, i| {
                acc * (Expression::Constant(F::from(i as u64)) - value.clone())
            })
        };

        let rank_check = |value: Expression<F>| {
            // Rank check Ace or 2 or 3 or 4 or 5 or 6 or 7 or 8 or 9
            //                or 10 or Jack or Queen or King          
            (1..12).fold(value.clone(), |acc, i| {
                acc * (Expression::Constant(F::from(i as u64)) - value.clone())
            })
        };

        // | a0  |  a1   |  a2  | selector |
        // |-----|-------|------|----------|        
        // | qty | suite | rank | s_check  |
        //  
        meta.create_gate("card check",
            |meta| {
                let s = meta.query_selector(s_check);
                
                // Private inputs
                let qty = meta.query_advice(qty, Rotation::cur());
                let suite = meta.query_advice(suite, Rotation::cur());
                let rank = meta.query_advice(rank, Rotation::cur());

                // Public inputs
                let pub_qty = meta.query_instance(pub_qty, Rotation::cur());
                let pub_suite = meta.query_instance(pub_suite, Rotation::cur());
                let pub_rank = meta.query_instance(pub_rank, Rotation::cur());

                // Card check is qty_check AND suite_check AND rank_check
                let qty_match = qty.clone() - pub_qty; 
                let suite_match = suite.clone() - pub_suite; 
                let rank_match = rank.clone() - pub_rank; 

                Constraints::with_selector(s,
                        [
                            ("qty_check", qty_check(qty)),
                            ("suite_check", suite_check(suite)),
                            ("rank_check", rank_check(rank)),
                            ("qty_match", qty_match),
                            ("suite_match", suite_match),
                            ("rank_match", rank_match),
                        ],
                )
        });

        //println!("\nCreategate done:{}", meta.get_instance_query_index(qty, Rotation::cur()));
        CardConfig {
            qty,
            suite,
            rank,
            pub_qty,
            pub_suite,
            pub_rank,
            s_check,
        }
    }

    pub fn assign(&self, mut layouter: impl Layouter<F>,
        qty: Value<Assigned<F>>, suite: Value<Assigned<F>>,
        rank: Value<Assigned<F>>) -> 
        Result<(CardConstrained<F>, CardConstrained<F>,
            CardConstrained<F>), Error> {

        let offset = 0;

        layouter.assign_region( || "Card", |mut region| {
            self.config.s_check.enable(&mut region, offset)?;

            let qty_cell = region
                .assign_advice(|| "qty value",
                self.config.qty, offset, || qty)
                .map(CardConstrained).unwrap();

            let suite_cell = region.
                assign_advice(|| "suite value",
                self.config.suite, offset, || suite)
                .map(CardConstrained).unwrap();

            let rank_cell: CardConstrained<F> = region.
                assign_advice(|| "rank value",
                self.config.rank, offset, || rank)
                .map(CardConstrained).unwrap();

            Ok((qty_cell, suite_cell, rank_cell))
        })
    }
}

#[derive(Default)]
struct CardCircuit<F: PrimeField> {
    qty: Value<Assigned<F>>,
    suite: Value<Assigned<F>>,
    rank: Value<Assigned<F>>,
}

impl<F: PrimeField> Circuit<F> for CardCircuit<F> {

    type Config = CardConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let qty = meta.advice_column();
        let suite = meta.advice_column();
        let rank = meta.advice_column();
        let pub_qty = meta.instance_column();
        let pub_suite = meta.instance_column();
        let pub_rank = meta.instance_column();
        let s_check = meta.selector();

        CardChip::configure(meta, qty, suite, rank,
                    pub_qty, pub_suite, pub_rank, s_check)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) 
        -> Result<(), Error> {

        let chip: CardChip<F> = CardChip::construct(config);

        let (qtycell, suitecell, rankcell) = chip
            .assign(layouter.namespace(|| "Card Assign"),
            self.qty, self.suite, self.rank).unwrap();

        Ok(())
    }
}

#[test]
fn test_range_check_1() {
    const k: u32 = 10;

    // Successful case
    let circuit = CardCircuit::<Fp> {
        qty: Value::known(Fp::from(1 as u64).into()),
        suite: Value::known(Fp::from(2 as u64).into()),
        rank: Value::known(Fp::from(3 as u64).into()),
    };

    let pubqty = Fp::from(1 as u64).into();
    let pubsuite = Fp::from(2 as u64).into();
    let pubrank = Fp::from(3 as u64).into();

    let public_inputs = vec![vec![pubqty], vec![pubsuite], vec![pubrank]];
    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
    prover.assert_satisfied();
}