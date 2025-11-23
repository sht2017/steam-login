{
    function sleep(ms) {
        return new Promise((r) => setTimeout(r, ms));
    }

    function setNativeValue(el, value) {
        const setter = Object.getOwnPropertyDescriptor(
            HTMLInputElement.prototype,
            "value"
        ).set;
        setter.call(el, value);
        el.dispatchEvent(new Event("input", { bubbles: true }));
    }

    // Wait for all required elements to be available in DOM
    async function waitForElements(timeout = 10000) {
        const startTime = Date.now();
        
        while (Date.now() - startTime < timeout) {
            const username = [...document.querySelectorAll("*")]
                .find(e => e.children.length === 0 && 
                      e.innerText?.toLowerCase().includes("sign in with account name"))
                ?.parentElement?.querySelector("input");
            
            const password = [...document.querySelectorAll("*")]
                .find(e => e.children.length === 0 && 
                      e.innerText?.toLowerCase().includes("password"))
                ?.parentElement?.querySelector("input");
            
            const button = [...document.querySelectorAll("button, input[type=button], input[type=submit]")]
                .find(el => el.innerText?.toLowerCase().includes("sign in"));
            
            // Return elements when all are found
            if (username && password && button) {
                return { username, password, button };
            }
            
            await sleep(50);
        }
        
        throw new Error("Timeout: Login elements not found");
    }

    (async () => {
        try {
            // Wait for all login elements to load
            const { username, password, button } = await waitForElements();

            // Fill username
            username.focus();
            setNativeValue(username, "{%username%}");
            username.blur();
            await sleep(100);

            // Fill password
            password.focus();
            setNativeValue(password, "{%password%}");
            password.blur();
            await sleep(100);

            // Click remember me checkbox if exists
            const remember_checkbox = [...document.querySelectorAll("*")]
                .find(e => e.innerText?.trim().toLowerCase() === "remember me")
                ?.childNodes?.[0];
            remember_checkbox?.click();

            // Click sign in button
            button.click();

            // Wait for and fill captcha inputs
            (async function fillCaptcha() {
                const code = "{%captcha%}";
                const inputs = [...document.querySelectorAll(".Panel input")]
                    .filter(i => i.offsetParent !== null);

                if (inputs.length >= code.length) {
                    for (let k = 0; k < code.length; k++) {
                        inputs[k].focus();
                        setNativeValue(inputs[k], code[k] || "");
                        await sleep(80);
                    }
                    return;
                }

                setTimeout(fillCaptcha, 50);
            })();
        } catch (error) {
            console.error("Steam login automation failed:", error);
        }
    })();
}
