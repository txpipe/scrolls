const BYRON_UNIX = 1506203091;
const SHELLY_UNIX = 1596491091;
const SHELLY_SLOT = 4924800;

export function slotToTimestamp(slotNumber: number) {
  let unixTimestamp;

  if(slotNumber <= SHELLY_SLOT) {
    unixTimestamp = BYRON_UNIX + slotNumber * 20
  } else {
    unixTimestamp = SHELLY_UNIX + (slotNumber - SHELLY_SLOT);
  }

  const date = new Date(unixTimestamp * 1000);
  
  return date.toISOString();
}