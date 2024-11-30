pub(crate) struct Stats {
    messages_sent: u64,
    fragments_sent: u64,

    messages_received: u64,
    fragments_received: u64,

    acks_sent: u64,
    acks_received: u64,

    nacks_received: u64,
}

// Setters
impl Stats {
    pub(crate) fn default() -> Self {
        Self {
            messages_sent: 0,
            fragments_sent: 0,

            messages_received: 0,
            fragments_received: 0,

            acks_sent: 0,
            acks_received: 0,

            nacks_received: 0,
        }
    }

    pub(crate) fn inc_messages_sent(&mut self) {
        self.messages_sent += 1;
    }

    pub(crate) fn inc_fragments_sent(&mut self) {
        self.fragments_sent += 1;
    }

    #[allow(dead_code)] // TODO: to be used in the future
    pub(crate) fn inc_messages_received(&mut self) {
        self.messages_received += 1;
    }

    pub(crate) fn inc_fragments_received(&mut self) {
        self.fragments_received += 1;
    }

    pub(crate) fn inc_acks_sent(&mut self) {
        self.acks_sent += 1;
    }

    pub(crate) fn inc_acks_received(&mut self) {
        self.acks_received += 1;
    }

    pub(crate) fn inc_nacks_received(&mut self) {
        self.nacks_received += 1;
    }
}

// Getters
impl Stats {
    pub fn get_messages_sent(&self) -> u64 {
        self.messages_sent
    }

    pub fn get_fragments_sent(&self) -> u64 {
        self.fragments_sent
    }

    pub fn get_messages_received(&self) -> u64 {
        self.messages_received
    }
  
    pub fn get_fragments_received(&self) -> u64 {
        self.fragments_received
    }
  
    pub fn get_acks_sent(&self) -> u64 {
        self.acks_sent
    }
  
    pub fn get_acks_received(&self) -> u64 {
        self.acks_received
    }
  
    pub fn get_nacks_received(&self) -> u64 {
      self.nacks_received
    }
}
