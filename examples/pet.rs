


#[derive(Debug)]
pub struct Dog(String);

impl Dog {
    fn say(&self) -> String {
        format!("{} says: Woof!", self.0)
    }
}

#[derive(Debug)]
pub struct Cat(String);

impl Cat {
    fn say(&self) -> String {
        format!("{} says: Meow!", self.0)
    }
}

#[derive(Debug)]
pub enum Pet {
    Dog(Dog),
    Cat(Cat),
}


#[interthread::actor(channel=2)]

impl Pet {

    pub fn new( pet: Self) -> Self {
        pet
    }

    pub fn speak(&self) -> String {
        match self {
           Self::Dog(dog) => {
            format!("Dog {}",dog.say())
            },
           Self::Cat(cat) => {
            format!("Cat {}", cat.say())
            },
        }
    }
    pub fn swap(&mut self, pet: Self ) -> Self {
        std::mem::replace(self,pet)
    }
}


fn main() {

    let pet = PetLive::new( 
        Pet::Dog(Dog("Tango".to_string()))
    );

    let mut pet_a = pet.clone();
    let pet_b     = pet.clone();
    
    let handle_a = std::thread::spawn( move || {
        println!("Thread A - {}",pet_a.speak());
        // swap the the pet 
        pet_a.swap(Pet::Cat(Cat("Kiki".to_string())))
    });

    let swapped_pet = handle_a.join().unwrap();

    let _handle_b = std::thread::spawn( move || {
        println!("Thread B - {}",pet_b.speak());
    }).join();

    //play with both pets now  
    println!("Thread MAIN - {}",pet.speak());
    println!("Thread MAIN - {}",swapped_pet.speak());

}

//outputs
/*
Thread A - Dog Tango says: Woof!
Thread B - Cat Kiki says: Meow!
Thread MAIN - Cat Kiki says: Meow!
Thread MAIN - Dog Tango says: Woof!
*/