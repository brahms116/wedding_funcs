DELETE FROM invitee;
INSERT INTO invitee (id, fname, lname, rsvp, dietary_requirements, invitation_opened) VALUES 
('2e53abf4-323a-48ce-9ec2-0c0850c15523', 'David', 'Kwong', 'Unknown', '', false),
('8a0bf7ec-4e44-4d2f-9a3f-bdb6e7bd3097', 'Mia', 'Huang', 'Unknown', '', false),
('0c6c0fc4-146f-4e2a-b081-7f3ec5281290', 'Joseph', 'Kwong', 'Unknown', '', false),
('e01601bb-0647-410f-91b5-ca4df097c175', 'Willian', 'Kwong', 'Unknown', '', false);


DELETE FROM relation;
INSERT INTO relation (parent,child) VALUES
('2e53abf4-323a-48ce-9ec2-0c0850c15523','8a0bf7ec-4e44-4d2f-9a3f-bdb6e7bd3097'),
('2e53abf4-323a-48ce-9ec2-0c0850c15523','0c6c0fc4-146f-4e2a-b081-7f3ec5281290');

