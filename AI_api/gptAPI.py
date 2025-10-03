from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel
import openai
import os


# Same structs as for rust
class SourceFile(BaseModel):
    filename: str
    content: str


class ReceivedPayload(BaseModel):
    user_id: str
    task: str
    read_me: str
    source_files: list[SourceFile]
    test_results: str


# Setting up api key(environment variable)
api_key = os.getenv("IMAGI_OPENAI_API_KEY")
if not api_key:
    raise RuntimeError("Missing IMAGI_OPENAI_API_KEY")

app = FastAPI()


# Error handling
@app.exception_handler(Exception)
async def generic_exception_handler(request: Request, exc: Exception):
    return JSONResponse(
        status_code=500, content={"detail": f"Internal server error: {str(exc)}"}
    )


# Main script for receiving requests, grading and sending back
#
# CHOOSE WHICH PROMPT YOU PREFER ON LINE 48!!!
@app.post("/imagi_gpt")
async def imagi(request: ReceivedPayload):
    try:
        openai.api_key = api_key

        filenames = [sf.filename for sf in request.source_files]
        contents = [sf.content for sf in request.source_files]

        filenames_str = ", ".join(filenames)
        contents_str = "\n\n".join(contents)

        with open("AI_api/student.txt") as f:
            template = f.read()
            prompt = template.format(
                request.read_me, filenames_str, contents_str, request.test_results
            )

        try:
            response = openai.chat.completions.create(
                model="gpt-4o-mini", messages=[{"role": "user", "content": prompt}]
            )
        except Exception as e:
            raise HTTPException(status_code=502, detail=f"OpenAI API error: {str(e)}")

        feedback = (
            response.choices[0].message.content
            if response.choices and response.choices[0].message.content
            else ""
        )
        if ":" not in feedback:
            raise HTTPException(
                status_code=500, detail="Malformed feedback format from OpenAI."
            )

        status, content = feedback.split(":", 1)
        status = status.strip()

        json_string = {
            "student_id": request.user_id,
            "task": request.task,
            "status": status,
            "feedback": content,
        }
        return json_string

    except HTTPException as e:
        # Re-raise HTTPExceptions so FastAPI can handle them
        raise e
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Internal error: {str(e)}")
