from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import base64, cv2, os, tempfile

app = FastAPI(title="Doctor AI Media Service")

class VideoRequest(BaseModel):
    video_base64: str
    max_frames: int = 8

@app.get("/health")
def health():
    return {"ok": True, "service": "doctor-ai-media"}

@app.post("/extract-frames")
def extract_frames(req: VideoRequest):
    raw = base64.b64decode(req.video_base64)
    suffix = ".mp4"
    with tempfile.NamedTemporaryFile(delete=False, suffix=suffix) as f:
        f.write(raw); path = f.name
    cap = cv2.VideoCapture(path)
    try:
        if not cap.isOpened():
            raise HTTPException(400, "Could not open video. Check codec/FFmpeg support.")
        total = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
        if total <= 0:
            raise HTTPException(400, "Video has no readable frames.")
        count = max(1, min(int(req.max_frames), 8))
        indices = sorted(set(round(i * (total - 1) / max(1, count - 1)) for i in range(count)))
        frames = []
        for idx in indices:
            cap.set(cv2.CAP_PROP_POS_FRAMES, idx)
            ok, frame = cap.read()
            if not ok: continue
            h, w = frame.shape[:2]
            scale = min(1.0, 1280 / max(w, h))
            if scale < 1:
                frame = cv2.resize(frame, (int(w*scale), int(h*scale)))
            ok, encoded = cv2.imencode(".jpg", frame, [int(cv2.IMWRITE_JPEG_QUALITY), 82])
            if ok: frames.append(base64.b64encode(encoded.tobytes()).decode("ascii"))
        return {"frames": frames, "total_frames": total, "sampled": len(frames)}
    finally:
        cap.release()
        os.unlink(path)
