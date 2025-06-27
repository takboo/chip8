<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { readFile } from "@tauri-apps/plugin-fs";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { onMount, onDestroy } from "svelte";

    interface EmulatorInfo {
        width: number;
        height: number;
        is_running: boolean;
    }

    interface FrameBuffer {
        data: number[];
        updated: boolean;
    }

    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D | null = null;
    let emulatorInfo: EmulatorInfo | null = null;
    let isInitialized = $state(false);
    let isRomLoaded = $state(false);
    let isRunning = $state(false);
    let showGrid = $state(true);
    let cpuSpeed = $state(500);
    let errorMessage = $state("");
    let isDragOver = $state(false);
    let showKeyboardHelp = $state(false);

    // Animation and rendering
    let animationId: number | null = null;
    let lastFrameTime = 0;
    const FPS = 60;
    const frameInterval = 1000 / FPS;

    // Key mapping for CHIP-8
    const keyMap: { [key: string]: number } = {
        "1": 0x1,
        "2": 0x2,
        "3": 0x3,
        "4": 0xc,
        q: 0x4,
        w: 0x5,
        e: 0x6,
        r: 0xd,
        a: 0x7,
        s: 0x8,
        d: 0x9,
        f: 0xe,
        z: 0xa,
        x: 0x0,
        c: 0xb,
        v: 0xf,
    };

    const pressedKeys = new Set<number>();

    async function initializeEmulator() {
        try {
            emulatorInfo = await invoke<EmulatorInfo>("initialize_emulator", {
                cpuSpeed: cpuSpeed,
            });
            isInitialized = true;
            errorMessage = "";
            setupCanvas();
        } catch (error) {
            errorMessage = `初始化失败: ${error}`;
            console.error("Failed to initialize emulator:", error);
        }
    }

    function setupCanvas() {
        if (!canvas || !emulatorInfo) return;

        ctx = canvas.getContext("2d");
        if (!ctx) return;

        // Set canvas internal resolution to match display size for crisp rendering
        const displayWidth = 640;
        const displayHeight = 320;
        canvas.width = displayWidth;
        canvas.height = displayHeight;

        // Configure rendering
        ctx.imageSmoothingEnabled = false;
        ctx.fillStyle = "#000000";
        ctx.fillRect(0, 0, canvas.width, canvas.height);
    }

    async function loadRomFile() {
        try {
            const filePath = await open({
                filters: [
                    { name: "CHIP-8 ROM", extensions: ["ch8", "c8"] },
                    { name: "All Files", extensions: ["*"] },
                ],
            });

            if (filePath) {
                const romData = await readFile(filePath as string);
                await invoke("load_rom", { romData: Array.from(romData) });
                isRomLoaded = true;
                errorMessage = "";
            }
        } catch (error) {
            errorMessage = `加载 ROM 失败: ${error}`;
            console.error("Failed to load ROM:", error);
        }
    }

    async function loadRomFromDrop(file: File) {
        try {
            console.log("Reading file:", file.name, "Size:", file.size);
            const arrayBuffer = await file.arrayBuffer();
            const romData = new Uint8Array(arrayBuffer);
            console.log("ROM data loaded, size:", romData.length);

            await invoke("load_rom", { romData: Array.from(romData) });
            isRomLoaded = true;
            errorMessage = "";
            console.log("ROM loaded successfully");
        } catch (error) {
            errorMessage = `Failed to load ROM: ${error}`;
            console.error("Failed to load ROM:", error);
        }
    }

    function startEmulation() {
        isRunning = true;
        gameLoop();
    }

    function stopEmulation() {
        isRunning = false;
        if (animationId) {
            cancelAnimationFrame(animationId);
            animationId = null;
        }
    }

    async function resetEmulator() {
        try {
            await invoke("reset_emulator");
            isRomLoaded = false;
            stopEmulation();
            if (ctx && emulatorInfo) {
                ctx.fillStyle = "#000000";
                ctx.fillRect(0, 0, emulatorInfo.width, emulatorInfo.height);
                drawGrid();
            }
        } catch (error) {
            errorMessage = `重置失败: ${error}`;
            console.error("Failed to reset:", error);
        }
    }

    async function updateCpuSpeed() {
        if (isInitialized) {
            try {
                await invoke("set_cpu_speed", { cpuSpeed: cpuSpeed });
            } catch (error) {
                console.error("Failed to update CPU speed:", error);
            }
        }
    }

    async function gameLoop() {
        if (!isRunning) return;

        const currentTime = performance.now();
        const deltaTime = currentTime - lastFrameTime;

        if (deltaTime >= frameInterval) {
            try {
                // Tick the emulator
                await invoke("tick_emulator");

                // Get framebuffer and render
                const frameBuffer =
                    await invoke<FrameBuffer>("get_framebuffer");
                if (frameBuffer.updated) {
                    renderFrame(frameBuffer.data);
                }

                lastFrameTime = currentTime;
            } catch (error) {
                console.error("Game loop error:", error);
                errorMessage = `运行错误: ${error}`;
                stopEmulation();
                return;
            }
        }

        animationId = requestAnimationFrame(gameLoop);
    }

    function renderFrame(framebuffer: number[]) {
        if (!ctx || !emulatorInfo) return;

        // Clear canvas with dark background
        ctx.fillStyle = "#0a0a0a";
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        // Calculate pixel size for scaling
        const pixelWidth = canvas.width / emulatorInfo.width;
        const pixelHeight = canvas.height / emulatorInfo.height;

        // Draw each CHIP-8 pixel as a scaled rectangle
        for (let y = 0; y < emulatorInfo.height; y++) {
            for (let x = 0; x < emulatorInfo.width; x++) {
                const pixelIndex = y * emulatorInfo.width + x;
                const isOn = framebuffer[pixelIndex] === 1;

                if (isOn) {
                    ctx.fillStyle = "#00ff64"; // Neon green for active pixels
                } else {
                    ctx.fillStyle = "#0a0a0a"; // Dark background for inactive pixels
                }

                ctx.fillRect(
                    x * pixelWidth,
                    y * pixelHeight,
                    pixelWidth,
                    pixelHeight,
                );
            }
        }

        if (showGrid) {
            drawGrid();
        }
    }

    function drawGrid() {
        if (!ctx || !emulatorInfo) return;

        ctx.strokeStyle = "rgba(0, 255, 100, 0.2)";
        ctx.lineWidth = 1;

        // Calculate pixel size for proper grid alignment
        const pixelWidth = canvas.width / emulatorInfo.width;
        const pixelHeight = canvas.height / emulatorInfo.height;

        // Draw vertical grid lines (one for each CHIP-8 pixel column)
        for (let x = 0; x <= emulatorInfo.width; x++) {
            const xPos = x * pixelWidth;
            ctx.beginPath();
            ctx.moveTo(xPos, 0);
            ctx.lineTo(xPos, canvas.height);
            ctx.stroke();
        }

        // Draw horizontal grid lines (one for each CHIP-8 pixel row)
        for (let y = 0; y <= emulatorInfo.height; y++) {
            const yPos = y * pixelHeight;
            ctx.beginPath();
            ctx.moveTo(0, yPos);
            ctx.lineTo(canvas.width, yPos);
            ctx.stroke();
        }
    }

    // Event handlers
    function handleKeyDown(event: KeyboardEvent) {
        const key = event.key.toLowerCase();

        // Handle help toggle (F1 or H key)
        if (key === "f1" || (key === "h" && !event.repeat)) {
            showKeyboardHelp = !showKeyboardHelp;
            event.preventDefault();
            return;
        }

        // Handle CHIP-8 keys
        if (key in keyMap && !pressedKeys.has(keyMap[key])) {
            pressedKeys.add(keyMap[key]);
            invoke("key_press", { key: keyMap[key] });
            event.preventDefault();
        }
    }

    function handleKeyUp(event: KeyboardEvent) {
        const key = event.key.toLowerCase();
        if (key in keyMap && pressedKeys.has(keyMap[key])) {
            pressedKeys.delete(keyMap[key]);
            invoke("key_release", { key: keyMap[key] });
            event.preventDefault();
        }
    }

    // Drag and drop handlers using Tauri API
    async function setupDragDrop() {
        const webview = getCurrentWebviewWindow();

        const unlisten = await webview.onDragDropEvent((event) => {
            console.log("Drag drop event:", event.payload);

            if (event.payload.type === "enter") {
                console.log(
                    "User entering drop zone with files:",
                    event.payload.paths,
                );
                isDragOver = true;
            } else if (event.payload.type === "over") {
                console.log("User hovering over drop zone");
                // Keep drag over state
            } else if (event.payload.type === "drop") {
                console.log("User dropped files:", event.payload.paths);
                isDragOver = false;
                handleFileDrop(event.payload.paths);
            } else if (event.payload.type === "leave") {
                console.log("Drag leave");
                isDragOver = false;
            }
        });

        return unlisten;
    }

    async function handleFileDrop(paths: string[]) {
        if (paths.length > 0) {
            const filePath = paths[0];
            console.log("Loading ROM from path:", filePath);

            if (isInitialized) {
                try {
                    // Read file using Tauri's fs API
                    const { readFile } = await import("@tauri-apps/plugin-fs");
                    const romData = await readFile(filePath);

                    console.log("ROM data loaded, size:", romData.length);
                    await invoke("load_rom", { romData: Array.from(romData) });
                    isRomLoaded = true;
                    errorMessage = "";
                    console.log("ROM loaded successfully");
                } catch (error) {
                    errorMessage = `Failed to load ROM: ${error}`;
                    console.error("Failed to load ROM:", error);
                }
            } else {
                errorMessage = "Please initialize emulator first";
            }
        }
    }

    let unlistenDragDrop: (() => void) | null = null;

    onMount(() => {
        document.addEventListener("keydown", handleKeyDown);
        document.addEventListener("keyup", handleKeyUp);

        // Setup Tauri drag and drop
        setupDragDrop().then((unlisten) => {
            unlistenDragDrop = unlisten;
        });

        return () => {
            document.removeEventListener("keydown", handleKeyDown);
            document.removeEventListener("keyup", handleKeyUp);
        };
    });

    onDestroy(() => {
        stopEmulation();
        if (unlistenDragDrop) {
            unlistenDragDrop();
        }
    });
</script>

<main class="container">
    <div class="header">
        <h1 class="title">CHIP-8 EMULATOR</h1>
        <div class="status-bar">
            <div class="status-indicator {isInitialized ? 'active' : ''}">
                <div class="indicator-dot"></div>
                <span>INITIALIZED</span>
            </div>
            <div class="status-indicator {isRomLoaded ? 'active' : ''}">
                <div class="indicator-dot"></div>
                <span>ROM LOADED</span>
            </div>
            <div class="status-indicator {isRunning ? 'active' : ''}">
                <div class="indicator-dot"></div>
                <span>RUNNING</span>
            </div>
        </div>
    </div>

    {#if errorMessage}
        <div class="error-message">
            <span class="error-icon">⚠</span>
            {errorMessage}
        </div>
    {/if}

    <div class="main-content">
        <div class="controls-panel">
            <div class="control-group">
                <label for="cpu-speed">CPU FREQUENCY (Hz)</label>
                <input
                    id="cpu-speed"
                    type="range"
                    min="100"
                    max="2000"
                    step="50"
                    bind:value={cpuSpeed}
                    onchange={updateCpuSpeed}
                />
                <span class="value-display">{cpuSpeed}</span>
            </div>

            <div class="button-group">
                {#if !isInitialized}
                    <button class="btn primary" onclick={initializeEmulator}>
                        <span class="btn-icon">⚡</span>
                        INITIALIZE EMULATOR
                    </button>
                {:else}
                    <button
                        class="btn secondary"
                        onclick={loadRomFile}
                        disabled={isRunning}
                    >
                        <span class="btn-icon">📁</span>
                        LOAD ROM
                    </button>

                    {#if isRomLoaded}
                        {#if !isRunning}
                            <button
                                class="btn success"
                                onclick={startEmulation}
                            >
                                <span class="btn-icon">▶</span>
                                START
                            </button>
                        {:else}
                            <button class="btn warning" onclick={stopEmulation}>
                                <span class="btn-icon">⏸</span>
                                PAUSE
                            </button>
                        {/if}
                    {/if}

                    <button class="btn danger" onclick={resetEmulator}>
                        <span class="btn-icon">🔄</span>
                        RESET
                    </button>
                {/if}
            </div>

            <div class="options">
                <label class="checkbox-container">
                    <input type="checkbox" bind:checked={showGrid} />
                    <span class="checkmark"></span>
                    SHOW GRID
                </label>

                <div class="help-hint">
                    <small>PRESS F1 OR H TO SHOW KEYBOARD MAPPING</small>
                </div>
            </div>
        </div>

        <div class="display-container">
            <div
                class="drop-zone {isDragOver ? 'drag-over' : ''}"
                role="application"
                aria-label="ROM file drag and drop area"
            >
                {#if isDragOver}
                    <div class="drop-overlay">
                        <div class="drop-content">
                            <span class="drop-icon">📱</span>
                            <p>DROP ROM FILES HERE</p>
                        </div>
                    </div>
                {/if}

                <canvas bind:this={canvas} class="display-canvas"></canvas>

                <div class="scanlines"></div>
            </div>
        </div>
    </div>

    <!-- 键盘帮助面板 -->
    {#if showKeyboardHelp}
        <div
            class="keyboard-help-overlay"
            role="dialog"
            aria-modal="true"
            aria-label="Keyboard mapping help"
            onclick={() => (showKeyboardHelp = false)}
            onkeydown={(e) => {
                if (e.key === "Escape") {
                    showKeyboardHelp = false;
                }
            }}
            tabindex="0"
        >
            <div
                class="keyboard-help-panel"
                role="document"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
            >
                <div class="help-header">
                    <h3>KEYBOARD MAPPING</h3>
                    <button
                        class="close-btn"
                        onclick={() => (showKeyboardHelp = false)}>×</button
                    >
                </div>

                <div class="help-content">
                    <div class="mapping-section">
                        <div class="chip8-layout">
                            <div class="layout-title">CHIP-8 KEYPAD</div>
                            <div class="key-grid-help">
                                <div class="key-row-help">
                                    {#each ["1", "2", "3", "C"] as key}
                                        <div class="key-button-help">{key}</div>
                                    {/each}
                                </div>
                                <div class="key-row-help">
                                    {#each ["4", "5", "6", "D"] as key}
                                        <div class="key-button-help">{key}</div>
                                    {/each}
                                </div>
                                <div class="key-row-help">
                                    {#each ["7", "8", "9", "E"] as key}
                                        <div class="key-button-help">{key}</div>
                                    {/each}
                                </div>
                                <div class="key-row-help">
                                    {#each ["A", "0", "B", "F"] as key}
                                        <div class="key-button-help">{key}</div>
                                    {/each}
                                </div>
                            </div>
                        </div>

                        <div class="arrow">→</div>

                        <div class="pc-layout">
                            <div class="layout-title">COMPUTER KEYBOARD</div>
                            <div class="key-grid-help">
                                <div class="key-row-help">
                                    {#each ["1", "2", "3", "4"] as key}
                                        <div
                                            class="key-button-help {pressedKeys.has(
                                                keyMap[key.toLowerCase()],
                                            )
                                                ? 'pressed'
                                                : ''}"
                                        >
                                            {key}
                                        </div>
                                    {/each}
                                </div>
                                <div class="key-row-help">
                                    {#each ["Q", "W", "E", "R"] as key}
                                        <div
                                            class="key-button-help {pressedKeys.has(
                                                keyMap[key.toLowerCase()],
                                            )
                                                ? 'pressed'
                                                : ''}"
                                        >
                                            {key}
                                        </div>
                                    {/each}
                                </div>
                                <div class="key-row-help">
                                    {#each ["A", "S", "D", "F"] as key}
                                        <div
                                            class="key-button-help {pressedKeys.has(
                                                keyMap[key.toLowerCase()],
                                            )
                                                ? 'pressed'
                                                : ''}"
                                        >
                                            {key}
                                        </div>
                                    {/each}
                                </div>
                                <div class="key-row-help">
                                    {#each ["Z", "X", "C", "V"] as key}
                                        <div
                                            class="key-button-help {pressedKeys.has(
                                                keyMap[key.toLowerCase()],
                                            )
                                                ? 'pressed'
                                                : ''}"
                                        >
                                            {key}
                                        </div>
                                    {/each}
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="help-shortcuts">
                        <h4>SHORTCUTS</h4>
                        <ul>
                            <li>
                                <kbd>F1</kbd> / <kbd>H</kbd>: SHOW/HIDE KEYBOARD
                                MAPPING
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    {/if}
</main>

<style>
    .container {
        padding: 20px;
        max-width: 1400px;
        margin: 0 auto;
        min-height: 100vh;
    }

    .header {
        text-align: center;
        margin-bottom: 30px;
    }

    .title {
        font-size: 3rem;
        margin: 0;
        text-shadow: 0 0 20px #00ff64;
        animation: glow 2s ease-in-out infinite alternate;
        font-family: var(--font-display);
        font-weight: 900;
        letter-spacing: 3px;
        text-transform: uppercase;
    }

    @keyframes glow {
        from {
            text-shadow:
                0 0 20px #00ff64,
                0 0 30px #00ff64,
                0 0 40px #00ff64;
        }
        to {
            text-shadow:
                0 0 10px #00ff64,
                0 0 20px #00ff64,
                0 0 30px #00ff64;
        }
    }

    .status-bar {
        display: flex;
        justify-content: center;
        gap: 30px;
        margin-top: 20px;
        font-family: var(--font-display);
        font-weight: 700;
        letter-spacing: 1px;
    }

    .status-indicator {
        display: flex;
        align-items: center;
        gap: 8px;
        opacity: 0.5;
        transition: opacity 0.3s ease;
        font-size: 0.9rem;
        text-transform: uppercase;
    }

    .status-indicator.active {
        opacity: 1;
    }

    .indicator-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: #666;
        transition: background 0.3s ease;
    }

    .status-indicator.active .indicator-dot {
        background: #00ff64;
        box-shadow: 0 0 10px #00ff64;
    }

    .error-message {
        background: rgba(255, 0, 50, 0.1);
        border: 1px solid rgba(255, 0, 50, 0.3);
        border-radius: 8px;
        padding: 15px;
        margin-bottom: 20px;
        color: #ff6b6b;
        display: flex;
        align-items: center;
        gap: 10px;
        font-family: var(--font-mono);
        font-weight: 500;
    }

    .error-icon {
        font-size: 1.2rem;
    }

    .main-content {
        display: grid;
        grid-template-columns: 300px 1fr;
        gap: 30px;
        margin-bottom: 30px;
    }

    .controls-panel {
        background: rgba(0, 255, 100, 0.05);
        border: 1px solid rgba(0, 255, 100, 0.2);
        border-radius: 12px;
        padding: 25px;
        backdrop-filter: blur(10px);
    }

    .control-group {
        margin-bottom: 25px;
    }

    .control-group label {
        display: block;
        margin-bottom: 10px;
        font-weight: 700;
        color: #00ff64;
        font-size: 1.1rem;
        font-family: var(--font-display);
        letter-spacing: 1px;
    }

    input[type="range"] {
        width: 100%;
        height: 6px;
        border-radius: 3px;
        background: #333;
        outline: none;
        appearance: none;
    }

    input[type="range"]::-webkit-slider-thumb {
        appearance: none;
        width: 18px;
        height: 18px;
        border-radius: 50%;
        background: #00ff64;
        cursor: pointer;
        box-shadow: 0 0 10px #00ff64;
    }

    .value-display {
        display: inline-block;
        margin-top: 5px;
        padding: 4px 8px;
        background: rgba(0, 255, 100, 0.1);
        border-radius: 4px;
        font-family: var(--font-mono);
        font-weight: 700;
        color: #00ff64;
        border: 1px solid rgba(0, 255, 100, 0.2);
    }

    .button-group {
        display: flex;
        flex-direction: column;
        gap: 12px;
        margin-bottom: 25px;
    }

    .btn {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        padding: 12px 20px;
        border: none;
        border-radius: 8px;
        font-family: var(--font-display);
        font-weight: 700;
        cursor: pointer;
        transition: all 0.3s ease;
        text-transform: uppercase;
        font-size: 0.9rem;
        letter-spacing: 1px;
    }

    .btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .btn.primary {
        background: linear-gradient(45deg, #00ff64, #00cc50);
        color: #000;
    }

    .btn.primary:hover:not(:disabled) {
        transform: translateY(-2px);
        box-shadow: 0 5px 15px rgba(0, 255, 100, 0.4);
    }

    .btn.secondary {
        background: rgba(0, 255, 100, 0.1);
        color: #00ff64;
        border: 1px solid rgba(0, 255, 100, 0.3);
    }

    .btn.success {
        background: rgba(0, 255, 0, 0.2);
        color: #00ff00;
        border: 1px solid #00ff00;
    }

    .btn.warning {
        background: rgba(255, 165, 0, 0.2);
        color: #ffa500;
        border: 1px solid #ffa500;
    }

    .btn.danger {
        background: rgba(255, 0, 0, 0.2);
        color: #ff6b6b;
        border: 1px solid #ff6b6b;
    }

    .options {
        border-top: 1px solid rgba(0, 255, 100, 0.2);
        padding-top: 20px;
    }

    .checkbox-container {
        display: flex;
        align-items: center;
        cursor: pointer;
        user-select: none;
        font-family: var(--font-display);
        font-weight: 700;
        letter-spacing: 1px;
        text-transform: uppercase;
    }

    .checkbox-container input {
        display: none;
    }

    .checkmark {
        width: 20px;
        height: 20px;
        border: 2px solid rgba(0, 255, 100, 0.5);
        border-radius: 4px;
        margin-right: 10px;
        transition: all 0.3s ease;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .checkbox-container input:checked + .checkmark {
        background: #00ff64;
        border-color: #00ff64;
    }

    .checkbox-container input:checked + .checkmark::after {
        content: "✓";
        color: #000;
        font-weight: bold;
        font-size: 14px;
    }

    .display-container {
        display: flex;
        justify-content: center;
        align-items: center;
        background: rgba(0, 0, 0, 0.8);
        border: 2px solid rgba(0, 255, 100, 0.3);
        border-radius: 12px;
        padding: 20px;
        position: relative;
        min-height: 400px;
    }

    .drop-zone {
        position: relative;
        width: 100%;
        height: 100%;
        display: flex;
        justify-content: center;
        align-items: center;
    }

    .drop-zone.drag-over {
        border-color: #00ff64 !important;
        background: rgba(0, 255, 100, 0.1);
    }

    .drop-zone.drag-over .display-canvas {
        border-color: #00ff64;
        box-shadow: 0 0 20px rgba(0, 255, 100, 0.3);
    }

    .drop-overlay {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 255, 100, 0.2);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 10;
        border-radius: 8px;
    }

    .drop-content {
        text-align: center;
        font-family: var(--font-display);
        font-weight: 700;
        letter-spacing: 1px;
    }

    .drop-icon {
        font-size: 3rem;
        display: block;
        margin-bottom: 10px;
    }

    .display-canvas {
        image-rendering: pixelated;
        image-rendering: -moz-crisp-edges;
        image-rendering: crisp-edges;
        width: 640px;
        height: 320px;
        border: 1px solid rgba(0, 255, 100, 0.5);
        background: #0a0a0a;
    }

    .scanlines {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: repeating-linear-gradient(
            0deg,
            transparent,
            transparent 2px,
            rgba(0, 255, 100, 0.03) 2px,
            rgba(0, 255, 100, 0.03) 4px
        );
        pointer-events: none;
    }

    .help-hint {
        margin-top: 15px;
        text-align: center;
        opacity: 0.7;
        font-family: var(--font-mono);
        font-size: 0.85rem;
    }

    .help-hint small {
        display: block;
        margin-bottom: 5px;
    }

    .keyboard-help-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.8);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        backdrop-filter: blur(5px);
    }

    .keyboard-help-panel {
        background: linear-gradient(
            135deg,
            rgba(26, 26, 46, 0.95),
            rgba(22, 33, 62, 0.95)
        );
        border: 2px solid rgba(0, 255, 100, 0.3);
        border-radius: 16px;
        padding: 30px;
        max-width: 800px;
        max-height: 80vh;
        overflow-y: auto;
        animation: slideIn 0.3s ease-out;
    }

    @keyframes slideIn {
        from {
            opacity: 0;
            transform: scale(0.9) translateY(-20px);
        }
        to {
            opacity: 1;
            transform: scale(1) translateY(0);
        }
    }

    .help-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 25px;
        border-bottom: 1px solid rgba(0, 255, 100, 0.2);
        padding-bottom: 15px;
    }

    .help-header h3 {
        margin: 0;
        color: #00ff64;
        font-size: 1.5rem;
        font-family: var(--font-display);
        font-weight: 900;
        letter-spacing: 2px;
    }

    .close-btn {
        background: none;
        border: none;
        color: #00ff64;
        font-size: 24px;
        cursor: pointer;
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 50%;
        transition: all 0.3s ease;
    }

    .close-btn:hover {
        background: rgba(0, 255, 100, 0.1);
        transform: scale(1.1);
    }

    .mapping-section {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 30px;
        margin-bottom: 30px;
    }

    .chip8-layout,
    .pc-layout {
        flex: 1;
    }

    .layout-title {
        text-align: center;
        font-weight: 700;
        margin-bottom: 15px;
        color: rgba(0, 255, 100, 0.9);
        text-transform: uppercase;
        font-size: 1rem;
        font-family: var(--font-display);
        letter-spacing: 1px;
    }

    .key-grid-help {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .key-row-help {
        display: flex;
        gap: 6px;
        justify-content: center;
    }

    .key-button-help {
        width: 40px;
        height: 40px;
        border: 1px solid rgba(0, 255, 100, 0.3);
        border-radius: 6px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: bold;
        transition: all 0.3s ease;
        background: rgba(0, 0, 0, 0.3);
        font-size: 0.9rem;
    }

    .key-button-help.pressed {
        background: rgba(0, 255, 100, 0.3);
        border-color: #00ff64;
        box-shadow: 0 0 10px rgba(0, 255, 100, 0.5);
        transform: scale(0.95);
    }

    .arrow {
        font-size: 2rem;
        color: rgba(0, 255, 100, 0.6);
        margin: 0 20px;
    }

    .help-shortcuts {
        border-top: 1px solid rgba(0, 255, 100, 0.2);
        padding-top: 20px;
    }

    .help-shortcuts h4 {
        margin: 0 0 15px 0;
        color: rgba(0, 255, 100, 0.8);
    }

    .help-shortcuts ul {
        list-style: none;
        padding: 0;
        margin: 0;
    }

    .help-shortcuts li {
        margin-bottom: 12px;
        display: flex;
        align-items: center;
        gap: 10px;
        color: rgba(0, 255, 100, 0.8);
        font-size: 0.95rem;
        font-family: var(--font-mono);
    }

    .help-shortcuts kbd {
        background: rgba(0, 255, 100, 0.1);
        border: 1px solid rgba(0, 255, 100, 0.3);
        border-radius: 4px;
        padding: 2px 6px;
        margin-right: 8px;
        font-family: monospace;
        font-size: 0.8rem;
    }

    @media (max-width: 1024px) {
        .main-content {
            grid-template-columns: 1fr;
            gap: 20px;
        }

        .display-canvas {
            width: 100%;
            max-width: 640px;
            height: auto;
            aspect-ratio: 2/1;
        }
    }
</style>
