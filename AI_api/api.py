from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel
import openai
import os

class SourceFile(BaseModel):
      filename: str
      content: str

class ReceivedPayload(BaseModel):
    user_id: str
    read_me: str
    source_files: list[SourceFile]
    test_results: str

api_key = os.getenv("GRADER_OPENAI_API_KEY")
if not api_key:
    raise RuntimeError("Missing OPENAI_API_KEY")

app = FastAPI()

@app.exception_handler(Exception)
async def generic_exception_handler(request: Request, exc: Exception):
    return JSONResponse(
        status_code=500,
        content={"detail": f"Internal server error: {str(exc)}"}
    )

@app.post("/grade")
async def grade(request: ReceivedPayload):
    try:
        openai.api_key = api_key

        filenames = [sf.filename for sf in request.source_files]
        contents = [sf.content for sf in request.source_files]

        filenames_str = ", ".join(filenames)
        contents_str = "\n\n".join(contents)

        prompt = f"""You are a Java expert and a teacher whose goal is to help students improve their Java programming skills.

        The task description is provided below. Students must follow these instructions to pass the assignment:
        Task description: {request.read_me}

        Students submit their solutions as code files.
        Here are the filenames: {filenames_str}
        Here is the content of each file: {contents_str}

        Some unit tests have been run to evaluate these files based on the given task.
        These tests were executed across all submitted files, and the results are provided here: {request.test_results}

        If the unit tests were successful, provide concise and constructive feedback. Focus on how the student can improve code readability, efficiency, structure, or other aspects of good software development practices.

        If the unit tests failed, guide the student toward identifying and understanding the problem.
        Do not state exactly what the issue is or how to fix it.
        Instead, ask guiding questions that encourage the student to reflect on their code’s behavior and discover the issue themselves.
        Do not give away the solution. Your role is purely pedagogical.

        Your feedback must follow this format:
        Pass/Fail (depending on the test results and your judgment)
        AI Feedback: <your feedback here>

        Example:
        Pass AI Feedback: Good solution. You could improve readability by using more descriptive variable names and simplifying your loop structure.

        Feedback must be short (no more than 100 words) and focused on helping the student become a better developer.

        It is very important to follow these instructions exactly, as your output will be used to grade students and will be processed into a JSON file.
        """

        try:
            response = openai.chat.completions.create(
                model = "gpt-4o-mini",
                messages = [{"role": "user", "content": prompt}]
            )
        except Exception as e:
            raise HTTPException(status_code=502, detail=f"OpenAI API error: {str(e)}")

        feedback = response.choices[0].message.content if response.choices and response.choices[0].message.content else ""
        if ":" not in feedback:
            raise HTTPException(status_code=500, detail="Malformed feedback format from OpenAI.")

        status, content = feedback.split(":", 1)
        status = status.strip()

        json_string = {"student_id": request.user_id, "status": status, "feedback": content}
        return json_string

    except HTTPException as e:
        # Re-raise HTTPExceptions so FastAPI can handle them
        raise e
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Internal error: {str(e)}")
