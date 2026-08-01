
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    PitchUp1 = 0,
    PitchUp7 = 1,
    PitchUp12 = 2,
    Up = 3,
    Down = 4,
    Left = 5,
    Right = 6,
}
impl TryFrom<u8> for Action {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Action::PitchUp1),
            1 => Ok(Action::PitchUp7),
            2 => Ok(Action::PitchUp12),
            3 => Ok(Action::Up),
            4 => Ok(Action::Down),
            5 => Ok(Action::Left),
            6 => Ok(Action::Right),
            _ => Err("Invaild action"),
        }
    }
}

pub struct Track {
    pub name: String,
}
impl Track {
    pub fn new(name: &str) -> Self {
        Self { name: name.into() }
    }
}

/// Internal encoded command.
///
/// upper 8 bits : track id
/// lower 8 bits : Action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Command(u16);

impl Command {
    #[inline]
    pub fn new(track_id: u16, action: Action) -> Self {
        Self((track_id << 8) | action as u16)
    }

    #[inline]
    pub fn into_u16(self) -> u16 {
        self.0
    }
}

impl From<Command> for u16 {
    fn from(value: Command) -> Self {
        value.0
    }
}


pub struct Y2kxFile {
    version: u8,

    title: String,
    artist: String,
    description: String,

    tracks: Vec<Track>,

    delays: Vec<u32>,              // uint24(ms)
    commands: Vec<Vec<Command>>,   // commands[i] belongs to delays[i]
}

impl Y2kxFile {
    // =====================================================
    // Constants
    // =====================================================

    const MAGIC: &[u8; 4] = b"y2kx";
    const HEADER_SIZE: usize = 5;

    // =====================================================
    // Constructors
    // =====================================================

    pub fn new(version: u8) -> Result<Self, &'static str> {
        if version != 0 {
            Err("Unsupported version")
        } else {
            Ok(Self { version, title: "".into(), artist: "".into(), description: "".into(), tracks: Vec::new(), delays: Vec::new(), commands: Vec::new() })
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < Self::HEADER_SIZE {
            return Err("The file is too short");
        }

        if &bytes[..4] != Self::MAGIC {
            return Err("Invalid magic");
        }

        let version = bytes[4];

        let mut file = Self::new(version)?;

        let mut body = &bytes[Self::HEADER_SIZE..];

        file.read_metadata(&mut body)?;
        file.read_tracks(&mut body)?;
        file.read_keyframes(body)?;

        Ok(file)
    }

    // =====================================================
    // Serialization
    // =====================================================
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();

        // -----------------------------
        // Header
        // -----------------------------
        out.extend_from_slice(Self::MAGIC);
        out.push(self.version);

        // Metadata
        out.extend_from_slice(self.title.as_bytes());
        out.push(0);

        out.extend_from_slice(self.artist.as_bytes());
        out.push(0);

        out.extend_from_slice(self.description.as_bytes());
        out.push(0);

        // -----------------------------
        // Tracks
        // -----------------------------
        out.push(
            self.tracks
                .len()
                .try_into()
                .expect("track count exceeds u8"),
        );

        for track in &self.tracks {
            out.extend_from_slice(track.name.as_bytes());
            out.push(0);
        }

        // -----------------------------
        // Keyframes
        // -----------------------------
        for i in 0..self.keyframe_count() {
            // uint24 delay
            assert!(self.delays[i] <= 0x00FF_FFFF);

            let bytes = self.delays[i].to_be_bytes();
            out.extend_from_slice(&bytes[1..]);

            // Commands
            for command in &self.commands[i] {
                out.extend_from_slice(&command.0.to_be_bytes());
            }

            // Terminator
            out.push(0);
        }

        out
    }

    // =====================================================
    // Metadata
    // =====================================================

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn set_title(&mut self, title: &str) -> Result<(), &'static str> {
        if title.contains('\0') {
            Err("Cannot contain '\\0'")
        }else {
            self.title = title.into();
            Ok(())
        }
    }

    pub fn artist(&self) -> &str {
        &self.artist
    }
    pub fn set_artist(&mut self, artist: &str) -> Result<(), &'static str> {
        if artist.contains('\0') {
            Err("Cannot contain '\\0'")
        }else {
            self.artist = artist.into();
            Ok(())
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn set_description(&mut self, description: &str) -> Result<(), &'static str> {
        if description.contains('\0') {
            Err("Cannot contain '\\0'")
        }else {
            self.description = description.into();
            Ok(())
        }
    }

    // =====================================================
    // Tracks
    // =====================================================

    pub fn tracks(&self) -> &Vec<Track> {
        &self.tracks
    }

    pub fn track_count(&self) -> u8 {
        self.tracks.len() as u8
    }

    pub fn add_track(&mut self, track: Track) -> Result<(), &'static str> {
        if self.tracks.len() >= 255 {
            Err("tracks length max is 255")
        } else {
            self.tracks.push(track);
            Ok(())
        }
    }

    pub fn reserve_tracks(&mut self, additional: usize) {
        self.tracks.reserve(additional);
    }

    // =====================================================
    // Keyframes
    // =====================================================

    pub fn keyframe_count(&self) -> usize {
        self.delays.len() // must equal to self.commands.len()
    }

    pub fn delays(&self) -> &Vec<u32> {
        &self.delays
    }

    pub fn commands(&self) -> &Vec<Vec<Command>> {
        &self.commands
    }

    pub fn reserve_keyframes(&mut self, additional: usize) {
        self.delays.reserve(additional);
        self.commands.reserve(additional);
    }

    pub fn add_keyframe(&mut self, delay: u32, commands: Vec<Command>) -> Result<(), &'static str> {
        Self::validate_delay(delay)?;
        for command in &commands {
            self.validate_command(*command)?;
        }

        self.delays.push(delay);
        self.commands.push(commands);
        
        Ok(())
    }

    pub fn add_keyframe_ref(&mut self, delay: u32, commands: &[Command]) -> Result<(), &'static str> {
        Self::validate_delay(delay)?;
        for command in commands {
            self.validate_command(*command)?;
        }

        self.delays.push(delay);
        self.commands.push(commands.to_vec());
        
        Ok(())
    }

    pub fn keyframe(&self, index: usize) -> Result<(u32, &[Command]), &'static str> {
        if index >= self.keyframe_count() {
            Err("index is out of range")
        } else {
            Ok((self.delays[index], &self.commands[index]))
        }
    }

    pub fn add_command(&mut self, keyframe: usize, command: Command) -> Result<(), &'static str> {
        if keyframe >= self.keyframe_count() {
            Err("index is out of range")
        } else {
            self.validate_command(command)?;
            self.commands[keyframe].push(command);
            Ok(())
        }
    }

    pub fn set_delay(&mut self, keyframe: usize, delay: u32) -> Result<(), &'static str> {
        if keyframe >= self.keyframe_count() {
            Err("index is out of range")
        } else {
            Self::validate_delay(delay)?;
            self.delays[keyframe] = delay;
            Ok(())
        }
    }

    pub fn remove_keyframe(&mut self, keyframe: usize) -> Result<(), &'static str> {
        if keyframe >= self.keyframe_count() {
            Err("index is out of range")
        } else {
            self.delays.remove(keyframe);
            self.commands.remove(keyframe);
            Ok(())
        }
    }

    // =====================================================
    // Validation
    // =====================================================

    fn validate_command(
        &self,
        command: Command,
    ) -> Result<(), &'static str> {
        let commands = command.into_u16().to_be_bytes();
        if commands[0] == 0 || commands[0] > self.track_count() {
            return Err("invaild instrument");
        }
        Action::try_from(commands[1])?;
        Ok(())
    }

    fn validate_delay(
        delay: u32,
    ) -> Result<(), &'static str> {
        if delay > 0xFFFFFF as u32 {
            Err("delay must be smaller then 0xFFFFFF")
        } else {
            Ok(())
        }
    }

    // =====================================================
    // Parser
    // =====================================================

    fn read_metadata(
        &mut self,
        body: &mut &[u8],
    ) -> Result<(), &'static str> {
        self.title = Self::read_cstring(body)?;
        self.artist = Self::read_cstring(body)?;
        self.description = Self::read_cstring(body)?;

        Ok(())
    }

    fn read_tracks(
        &mut self,
        body: &mut &[u8],
    ) -> Result<(), &'static str> {
        if body.is_empty() {
            return Err("Unexpected end of file");
        }

        let track_count = body[0] as usize;
        *body = &body[1..];

        self.reserve_tracks(track_count);

        for _ in 0..track_count {
            let name = Self::read_cstring(body)?;
            self.add_track(Track::new(&name))?;
        }

        Ok(())
    }
    fn read_keyframes(
        &mut self,
        body: &[u8],
    ) -> Result<(), &'static str> {
        let mut body = body;

        while !body.is_empty() {
            // Delay (uint24)
            if body.len() < 3 {
                return Err("Unexpected end of file");
            }

            let delay =
                ((body[0] as u32) << 16)
                | ((body[1] as u32) << 8)
                | (body[2] as u32);

            body = &body[3..];

            let mut commands = Vec::new();

            loop {
                if body.is_empty() {
                    return Err("Unexpected end of file");
                }

                // Terminator
                if body[0] == 0 {
                    body = &body[1..];
                    break;
                }

                if body.len() < 2 {
                    return Err("Unexpected end of file");
                }

                let command = Command(
                    u16::from_be_bytes([body[0], body[1]])
                );

                self.validate_command(command)?;
                commands.push(command);

                body = &body[2..];
            }

            self.add_keyframe(delay, commands)?;
        }

        Ok(())
    }

    fn read_cstring(body: &mut &[u8]) -> Result<String, &'static str> {
        let end = body
            .iter()
            .position(|&b| b == 0)
            .ok_or("Missing string terminator")?;

        let string = String::from_utf8_lossy(&body[..end]).into_owned();
        *body = &body[end + 1..];

        Ok(string)
    }
}