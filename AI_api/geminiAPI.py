from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel
import os
from google import genai

# --- Data structures ---
class SourceFile(BaseModel):
    filename: str
    content: str

class ReceivedPayload(BaseModel):
    user_id: str
    task: str
    read_me: str
    source_files: list[SourceFile]
    test_results: str

# --- API Key ---
api_key = os.getenv("GRADER_GEMINI_API_KEY")
if not api_key:
    raise RuntimeError("Missing GRADER_GEMINI_API_KEY")

# --- FastAPI app ---
app = FastAPI()

# --- Error handling ---
@app.exception_handler(Exception)
async def generic_exception_handler(request: Request, exc: Exception):
    return JSONResponse(
        status_code=500,
        content={"detail": f"Internal server error: {str(exc)}"}
    )

# --- Main grading endpoint ---
@app.post("/grade_gemini")
async def grade(request: ReceivedPayload):
    try:
        client = genai.Client(api_key=api_key)

        # Gather filenames and contents
        filenames = [sf.filename for sf in request.source_files]
        contents = [sf.content for sf in request.source_files]

        filenames_str = ", ".join(filenames)
        contents_str = "\n\n".join(contents)

        # Build prompt using teacher.txt template
        with open("teacher.txt") as f:
            template = f.read()
            prompt = template.format(
                request.read_me,
                filenames_str,
                contents_str,
                request.test_results
            )

        try:
            response = client.models.generate_content(
                model="gemini-2.5-flash",
                contents=prompt
            )
        except Exception as e:
            raise HTTPException(status_code=502, detail=f"Gemini API error: {str(e)}")

        feedback = response.text.strip() if response.text else ""
        if ":" not in feedback:
            raise HTTPException(status_code=500, detail="Malformed feedback format from Gemini (expected 'status: feedback').")

        status, content = feedback.split(":", 1)
        status = status.strip()

        json_string = {
            "student_id": request.user_id,
            "task": request.task,
            "status": status,
            "feedback": content.strip()
        }
        return json_string

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Internal error: {str(e)}")
