DROP TABLE IF EXISTS invitee CASCADE;
CREATE TABLE invitee (
  id TEXT UNIQUE NOT NULL PRIMARY KEY,
  fname TEXT NOT NULL,
  lname TEXT NOT NULL,
  rsvp TEXT NOT NULL,
  dietary_requirements TEXT NOT NULL,
  invitation_opened BOOL NOT NULL
);

DROP TABLE IF EXISTS relation CASCADE;
CREATE TABLE relation (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  parent TEXT NOT NULL REFERENCES invitee(id) ON UPDATE CASCADE ON DELETE CASCADE,
  child TEXT NOT NULL REFERENCES invitee(id) ON UPDATE CASCADE ON DELETE CASCADE
);

DROP TABLE IF EXISTS email CASCADE;
CREATE TABLE email (
  invitee TEXT NOT NULL PRIMARY KEY REFERENCES invitee(id) ON UPDATE CASCADE ON DELETE CASCADE,
  email TEXT NOT NULL,
  inite_sent BOOL NOT NULL DEFAULT FALSE
);


