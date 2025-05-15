pub struct VisibilitySystem {}

// FOV system
// The System is asking us what it needs to be done
// 'a specifies that the lifetime must be long enough to make the System run
impl<'a> System<'a> for VisibilitySystem {
    // SystemData is an alias of the tuple (ReadStorage, WriteStorage)
    // Here we define what kind of SystemData the "run" function will use and how
    //Get all Entities with Viewshed and Position Components (Read and Write)
    type SystemData = (WriteStorage<'a, Viewshed>, WriteStorage<'a, Position>);

    // System Trait implementation
    // We get the data we defined as SystemData (readalbe LeftMover components and writeable Position components)
    // and do stuff with it
    fn run(&mut self, (mut Viewshed, pos): Self::SystemData) {
        
        //For each one that has both Components
        for (viewshed, pos) in (&mut viewshed, &pos).join() {}
    }
}
