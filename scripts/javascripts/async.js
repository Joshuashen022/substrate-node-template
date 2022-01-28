function abcAfter2Seconds() {
    return new Promise(abc => {
        console.log('calling before setTimeout');
        setTimeout(() => {
            console.log('calling before resolved');
            abc('resolved');
            console.log('calling after resolved');
        }, 2000);

        console.log('calling after setTimeout');
    });
}
  
async function asyncCall() {
    console.log('calling1');
    const result = await abcAfter2Seconds();
    console.log('calling2');
    // console.log(result);
    // expected output: "resolved"
}
  
asyncCall();