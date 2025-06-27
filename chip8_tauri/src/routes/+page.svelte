<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { readFile } from "@tauri-apps/plugin-fs";
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
    "1": 0x1, "2": 0x2, "3": 0x3, "4": 0xC,
    "q": 0x4, "w": 0x5, "e": 0x6, "r": 0xD,
    "a": 0x7, "s": 0x8, "d": 0x9, "f": 0xE,
    "z": 0xA, "x": 0x0, "c": 0xB, "v": 0xF
  };

  const pressedKeys = new Set<number>();

  async function initializeEmulator() {
    try {
      emulatorInfo = await invoke<EmulatorInfo>("initialize_emulator", { 
        cpuSpeed: cpuSpeed 
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

    // Set canvas size
    canvas.width = emulatorInfo.width;
    canvas.height = emulatorInfo.height;
    
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
          { name: "All Files", extensions: ["*"] }
        ]
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
      const arrayBuffer = await file.arrayBuffer();
      const romData = new Uint8Array(arrayBuffer);
      await invoke("load_rom", { romData: Array.from(romData) });
      isRomLoaded = true;
      errorMessage = "";
    } catch (error) {
      errorMessage = `加载 ROM 失败: ${error}`;
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
        const frameBuffer = await invoke<FrameBuffer>("get_framebuffer");
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

    // Clear canvas
    ctx.fillStyle = "#0a0a0a";
    ctx.fillRect(0, 0, emulatorInfo.width, emulatorInfo.height);

    // Draw pixels
    const imageData = ctx.createImageData(emulatorInfo.width, emulatorInfo.height);
    
    for (let i = 0; i < framebuffer.length; i++) {
      const pixelIndex = i * 4;
      const color = framebuffer[i] === 1 ? 255 : 0;
      
      // Neon green for active pixels
      if (color === 255) {
        imageData.data[pixelIndex] = 0;     // R
        imageData.data[pixelIndex + 1] = 255; // G
        imageData.data[pixelIndex + 2] = 100; // B
      } else {
        imageData.data[pixelIndex] = 10;    // R
        imageData.data[pixelIndex + 1] = 10; // G
        imageData.data[pixelIndex + 2] = 10; // B
      }
      imageData.data[pixelIndex + 3] = 255; // A
    }

    ctx.putImageData(imageData, 0, 0);
    
    if (showGrid) {
      drawGrid();
    }
  }

  function drawGrid() {
    if (!ctx || !emulatorInfo) return;

    ctx.strokeStyle = "rgba(0, 255, 100, 0.15)";
    ctx.lineWidth = 0.5;

    // Calculate pixel size in canvas coordinates
    const canvasElement = canvas;
    const pixelWidth = canvasElement.width / emulatorInfo.width;
    const pixelHeight = canvasElement.height / emulatorInfo.height;

    // Draw grid lines for each pixel
    // Vertical lines
    for (let x = 0; x <= emulatorInfo.width; x++) {
      ctx.beginPath();
      ctx.moveTo(x * pixelWidth, 0);
      ctx.lineTo(x * pixelWidth, canvasElement.height);
      ctx.stroke();
    }

    // Horizontal lines
    for (let y = 0; y <= emulatorInfo.height; y++) {
      ctx.beginPath();
      ctx.moveTo(0, y * pixelHeight);
      ctx.lineTo(canvasElement.width, y * pixelHeight);
      ctx.stroke();
    }
  }

  // Event handlers
  function handleKeyDown(event: KeyboardEvent) {
    const key = event.key.toLowerCase();
    
    // Handle help toggle (F1 or H key)
    if (key === 'f1' || (key === 'h' && !event.repeat)) {
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

  // Drag and drop handlers
  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    event.dataTransfer!.dropEffect = "copy";
    isDragOver = true;
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    // Only hide if we're leaving the drop zone entirely
    if (!event.currentTarget?.contains(event.relatedTarget as Node)) {
      isDragOver = false;
    }
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;

    const files = event.dataTransfer?.files;
    if (files && files.length > 0) {
      if (isInitialized) {
        await loadRomFromDrop(files[0]);
      } else {
        errorMessage = "请先初始化模拟器";
      }
    }
  }

  onMount(() => {
    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("keyup", handleKeyUp);
    
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("keyup", handleKeyUp);
    };
  });

  onDestroy(() => {
    stopEmulation();
  });
</script>

<main class="container">
  <div class="header">
    <h1 class="title">CHIP-8 模拟器</h1>
    <div class="status-bar">
      <div class="status-indicator {isInitialized ? 'active' : ''}">
        <div class="indicator-dot"></div>
        <span>系统状态</span>
      </div>
      <div class="status-indicator {isRomLoaded ? 'active' : ''}">
        <div class="indicator-dot"></div>
        <span>ROM 已加载</span>
      </div>
      <div class="status-indicator {isRunning ? 'active' : ''}">
        <div class="indicator-dot"></div>
        <span>运行中</span>
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
        <label for="cpu-speed">CPU 频率 (Hz)</label>
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
            初始化模拟器
          </button>
        {:else}
          <button class="btn secondary" onclick={loadRomFile} disabled={isRunning}>
            <span class="btn-icon">📁</span>
            加载 ROM
          </button>
          
          {#if isRomLoaded}
            {#if !isRunning}
              <button class="btn success" onclick={startEmulation}>
                <span class="btn-icon">▶</span>
                开始
              </button>
            {:else}
              <button class="btn warning" onclick={stopEmulation}>
                <span class="btn-icon">⏸</span>
                暂停
              </button>
            {/if}
          {/if}

          <button class="btn danger" onclick={resetEmulator}>
            <span class="btn-icon">🔄</span>
            重置
          </button>
        {/if}
      </div>

      <div class="options">
        <label class="checkbox-container">
          <input type="checkbox" bind:checked={showGrid} />
          <span class="checkmark"></span>
          显示网格线
        </label>
        
        <div class="help-hint">
          <small>按 F1 或 H 键显示键盘映射</small>
        </div>
      </div>
    </div>

    <div class="display-container">
      <div 
        class="drop-zone {isDragOver ? 'drag-over' : ''}"
        ondragover={handleDragOver}
        ondragleave={handleDragLeave}
        ondrop={handleDrop}
      >
        {#if isDragOver}
          <div class="drop-overlay">
            <div class="drop-content">
              <span class="drop-icon">📱</span>
              <p>拖放 ROM 文件到这里</p>
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
    <div class="keyboard-help-overlay" onclick={() => showKeyboardHelp = false}>
      <div class="keyboard-help-panel" onclick={(e) => e.stopPropagation()}>
        <div class="help-header">
          <h3>键盘映射</h3>
          <button class="close-btn" onclick={() => showKeyboardHelp = false}>×</button>
        </div>
        
        <div class="help-content">
          <div class="mapping-section">
            <div class="chip8-layout">
              <div class="layout-title">CHIP-8 键盘</div>
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
              <div class="layout-title">计算机键盘</div>
              <div class="key-grid-help">
                <div class="key-row-help">
                  {#each ["1", "2", "3", "4"] as key}
                    <div class="key-button-help {pressedKeys.has(keyMap[key.toLowerCase()]) ? 'pressed' : ''}">{key}</div>
                  {/each}
                </div>
                <div class="key-row-help">
                  {#each ["Q", "W", "E", "R"] as key}
                    <div class="key-button-help {pressedKeys.has(keyMap[key.toLowerCase()]) ? 'pressed' : ''}">{key}</div>
                  {/each}
                </div>
                <div class="key-row-help">
                  {#each ["A", "S", "D", "F"] as key}
                    <div class="key-button-help {pressedKeys.has(keyMap[key.toLowerCase()]) ? 'pressed' : ''}">{key}</div>
                  {/each}
                </div>
                <div class="key-row-help">
                  {#each ["Z", "X", "C", "V"] as key}
                    <div class="key-button-help {pressedKeys.has(keyMap[key.toLowerCase()]) ? 'pressed' : ''}">{key}</div>
                  {/each}
                </div>
              </div>
            </div>
          </div>
          
          <div class="help-shortcuts">
            <h4>快捷键</h4>
            <ul>
              <li><kbd>F1</kbd> 或 <kbd>H</kbd> - 显示/隐藏此帮助面板</li>
              <li><kbd>Esc</kbd> - 退出应用</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  :global(body) {
    background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #16213e 100%);
    color: #00ff64;
    font-family: 'Courier New', monospace;
    margin: 0;
    padding: 0;
    min-height: 100vh;
  }

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
  }

  @keyframes glow {
    from {
      text-shadow: 0 0 20px #00ff64, 0 0 30px #00ff64, 0 0 40px #00ff64;
    }
    to {
      text-shadow: 0 0 10px #00ff64, 0 0 20px #00ff64, 0 0 30px #00ff64;
    }
  }

  .status-bar {
    display: flex;
    justify-content: center;
    gap: 30px;
    margin-top: 20px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    opacity: 0.5;
    transition: opacity 0.3s ease;
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
    background: rgba(255, 0, 0, 0.1);
    border: 1px solid #ff0000;
    border-radius: 8px;
    padding: 15px;
    margin-bottom: 20px;
    display: flex;
    align-items: center;
    gap: 10px;
    color: #ff6b6b;
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
    margin-bottom: 8px;
    font-weight: bold;
    text-transform: uppercase;
    font-size: 0.9rem;
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
    font-family: monospace;
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
    font-family: inherit;
    font-weight: bold;
    cursor: pointer;
    transition: all 0.3s ease;
    text-transform: uppercase;
    font-size: 0.9rem;
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
  }

  .help-hint small {
    font-size: 0.8rem;
    color: rgba(0, 255, 100, 0.8);
  }

  /* 键盘帮助面板样式 */
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
    background: linear-gradient(135deg, rgba(26, 26, 46, 0.95), rgba(22, 33, 62, 0.95));
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
    text-shadow: 0 0 10px rgba(0, 255, 100, 0.5);
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

  .chip8-layout, .pc-layout {
    flex: 1;
  }

  .layout-title {
    text-align: center;
    font-weight: bold;
    margin-bottom: 15px;
    color: rgba(0, 255, 100, 0.8);
    text-transform: uppercase;
    font-size: 0.9rem;
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
    margin-bottom: 8px;
    display: flex;
    align-items: center;
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
