export async function reduce(block) {
  let crtdCommands = [
    {
      SetAdd: [
        "2374a647493879459f0f64ffdc05cebb9b15f2a5b77a5a5ee418561497d2e200#0",
        '{"address":"addr1z8snz7c4974vzdpxu65ruphl3zjdvtxw8strf2c2tmqnxz2j2c79gy9l76sdg0xwhd7r0c0kna0tycz4y5s6mlenh8pq0xmsha","amount":[{"quantity":"281701032796","unit":"lovelace"},{"quantity":"1","unit":"2ae01189d7a9e539f05f2f6daa803ddcffff95ef9130215f483189866a643ec6"},{"quantity":"1","unit":"4d494e53574150"},{"quantity":"85604230567094","unit":"50617269627573"},{"quantity":"268994534","unit":"2ae01189d7a9e539f05f2f6daa803ddcffff95ef9130215f483189866a643ec6"}],"datum":"d8799fd8799f4040ffd8799f581ccc8d1b026353022abbfcc2e1e71159f9e308d9c6e905ac1db24c7fb64750617269627573ff1b0000035b71293bdc1b0000047751f613b1d8799fd8799fd8799fd8799f581caafb1196434cb837fd6f21323ca37b302dff6387e8a84b3fa28faf56ffd8799fd8799fd8799f581c52563c5410bff6a0d43ccebb7c37e1f69f5eb260552521adff33b9c2ffffffffd87a80ffffff"}',
      ],
    },
    {
      SetAdd: [
        "2374a647493879459f0f64ffdc05cebb9b15f2a5b77a5a5ee418561497d2e200#1",
        '{"address":"addr1z8snz7c4974vzdpxu65ruphl3zjdvtxw8strf2c2tmqnxz2j2c79gy9l76sdg0xwhd7r0c0kna0tycz4y5s6mlenh8pq0xmsha","amount":[{"quantity":"281701032796","unit":"lovelace"},{"quantity":"1","unit":"2ae01189d7a9e539f05f2f6daa803ddcffff95ef9130215f483189866a643ec6"},{"quantity":"1","unit":"4d494e53574150"},{"quantity":"85604230567094","unit":"50617269627573"},{"quantity":"268994534","unit":"2ae01189d7a9e539f05f2f6daa803ddcffff95ef9130215f483189866a643ec6"}],"datum":"d8799fd8799f4040ffd8799f581ccc8d1b026353022abbfcc2e1e71159f9e308d9c6e905ac1db24c7fb64750617269627573ff1b0000035b71293bdc1b0000047751f613b1d8799fd8799fd8799fd8799f581caafb1196434cb837fd6f21323ca37b302dff6387e8a84b3fa28faf56ffd8799fd8799fd8799f581c52563c5410bff6a0d43ccebb7c37e1f69f5eb260552521adff33b9c2ffffffffd87a80ffffff"}',
      ],
    },
  ];

  return crtdCommands;
}
