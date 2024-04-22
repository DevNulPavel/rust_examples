#! /usr/bin/env python3
# -*- coding: utf-8 -*-

import os.path
import getopt
import sys
import re
import tempfile
import platform
import subprocess
import shutil
import threading
import concurrent.futures
import multiprocessing


RE1 = re.compile("\/\*.+\*\/")
RE2 = re.compile("\/\*.+\*\/\n")
RE3 = re.compile("\/\/.+\n")
RE4 = re.compile("void main\s*\(void\)\s*{[\s\S=]+}", flags=re.MULTILINE)
RE5 = re.compile("void main\s*\(void\)\s*{", flags=0)

def convertTypeToSize(typeName):
    if typeName == "bool":
        return 1
    elif typeName == "int":
        return 4
    elif typeName == "float":
        return 4
    elif typeName == "vec2":
        return 4*2
    elif typeName == "vec3":
        return 4*3
    elif typeName == "vec4":
        return 4*4
    elif typeName == "mat2":
        return 4*2*2
    elif typeName == "mat3":
        return 4*3*3
    elif typeName == "mat4":
        return 4*4*4
    else:
        print("Invalid constant buffer type: %s" % typeName)
        sys.exit(2)

def convertTypeToSortPriority(typeName):
    if typeName == "bool":
        return 1
    elif typeName == "int":
        return 2
    elif typeName == "float":
        return 3
    elif typeName == "vec2":
        return 4
    elif typeName == "vec3":
        return 5
    elif typeName == "vec4":
        return 6
    elif typeName == "mat2":
        return 7
    elif typeName == "mat3":
        return 8
    elif typeName == "mat4":
        return 9
    else:
        print("Invalid constant buffer type: %s" % typeName)
        sys.exit(2)


def analyseUniformsSize(filePath) -> (int, dict):
    with open(filePath, "r") as file:
        inputFileText = file.read()

    # Табы заменяем на пробелы
    inputFileText = inputFileText.replace("\t", "    ")

    # Удаляем комментарии
    inputFileText = RE1.sub("", inputFileText) #re.DOTALL
    inputFileText = RE2.sub("\n", inputFileText)
    inputFileText = RE3.sub("\n", inputFileText)

    # Удаляем странные дефайны
    inputFileText = inputFileText.replace("FLOAT", "float")

    # Удаляем пустые строки
    # inputFileText = re.sub("^\s*$", "", inputFileText, flags=re.MULTILINE)

    words = inputFileText.replace("\n", " ").split(" ")
    words = list(filter(lambda a: a != "", words))

    # Main function handle
    mainFunctionText = RE4.search(inputFileText).group(0)

    # Remove precision words
    precisionWords = ["lowp", "mediump", "highp", "PRECISION_LOW", "PRECISION_MEDIUM", "PRECISION_HIGH", "PLATFORM_PRECISION"]
    for precisionWord in precisionWords:
        mainFunctionText = mainFunctionText.replace(precisionWord, "PRECISION")
    mainFunctionText = mainFunctionText.replace("PRECISION ", "")

    testUniformsNamesList = []
    uniformsDict = {}

    # Обходим слова и ищем аттрибуты
    uniformsSize = 0
    i = 0
    while i < len(words):
        if words[i] == "uniform":
            # TODO: ???
            # Если после uniform идет описание точности - пропускаем его
            i += 1
            if words[i] in precisionWords:
                i += 1

            # Если после uniform идет sampler2D - не обрабоатываем
            if words[i] != "sampler2D":
                # Получаем тип аттрибута
                testUniformType = words[i]

                # Получаем имя аттрибута
                i += 1
                testUniformName = words[i].replace(";", "")

                if (testUniformName not in testUniformsNamesList) and (testUniformName in mainFunctionText):
                    uniformsSize += convertTypeToSize(testUniformType)
                    testUniformsNamesList.append(testUniformName)
                    uniformsDict[testUniformName] = testUniformType
        i += 1

    return uniformsSize, uniformsDict


def processShaderFile(isVertexShader, inputPath, outputPath, setIndex, inputVaryingLocations, fullUniformsDict, pushConstantOffset, usePushConstant) -> (int, int, dict):
    with open(inputPath, "r") as file:
        inputFileText = file.read()

    # Табы заменяем на пробелы
    inputFileText = inputFileText.replace("\t", "    ")

    # Удаляем комментарии
    inputFileText = RE1.sub("", inputFileText) #re.DOTALL
    inputFileText = RE2.sub("\n", inputFileText)
    inputFileText = RE3.sub("\n", inputFileText)

    # Удаляем странные дефайны
    inputFileText = inputFileText.replace("FLOAT", "float")

    # Удаляем пустые строки
    # inputFileText = re.sub("^\s*$", "", inputFileText, flags=re.MULTILINE)

    words = inputFileText.replace("\n", " ").split(" ")
    words = list(filter(lambda a: a != "", words))

    # Main function handle
    mainFunctionText = RE4.search(inputFileText).group(0)

    # Remove precision words
    precisionWords = ["lowp", "mediump", "highp", "PRECISION_LOW", "PRECISION_MEDIUM", "PRECISION_HIGH", "PLATFORM_PRECISION"]
    for precisionWord in precisionWords:
        mainFunctionText = mainFunctionText.replace(precisionWord, "PRECISION")
    mainFunctionText = mainFunctionText.replace("PRECISION ", "")

    # Ignore defines
    ignoreDefinesList = ["float", "FLOAT", "VEC2"]

    # Lists
    definesNamesList = []
    definesMap = {}
    attributesNamesList = []
    attributesMap = {}
    uniformsNamesList = []
    uniformsMap = {}
    varyingsNamesList = []
    varyingsMap = {}
    samplersNamesList = []
    resultVaryingLocations = {}

    # Обходим слова и ищем аттрибуты
    attributeIndex = 0
    varyingIndex = 0
    i = 0
    while i < len(words):
        if words[i] == "#define":
            # Получаем тип аттрибута
            i += 1
            defineName = words[i]

            if (defineName in mainFunctionText) and (defineName not in ignoreDefinesList):
                # Получаем значение
                i += 1
                defineValue = words[i]

                # Добавляем аттрибут к тексту нового шейдера
                if defineName not in definesNamesList:
                    newShaderDefineName = "#define %s %s\n" % (defineName, defineValue)
                    definesMap[defineName] = newShaderDefineName
                    definesNamesList.append(defineName)

        if words[i] == "attribute":
            # TODO: ???
            # Если после attribute идет описание точности - пропускаем его
            i += 1
            if words[i] in precisionWords:
                i += 1

            # Получаем тип аттрибута
            attributeType = words[i]

            # Получаем имя аттрибута
            i += 1
            attributeName = words[i].replace(";", "")

            # Добавляем аттрибут к тексту нового шейдера
            if attributeName not in attributesMap:
                newShaderVariableName = "layout(location = %d) in %s %s;\n" % (attributeIndex, attributeType, attributeName)
                attributesMap[attributeName] = newShaderVariableName
                attributesNamesList.append(attributeName)
                attributeIndex += 1

        if words[i] == "uniform":
            # TODO: ???
            # Если после uniform идет описание точности - пропускаем его
            i += 1
            if words[i] in precisionWords:
                i += 1

            # Если после uniform идет sampler2D - не обрабоатываем
            if words[i] != "sampler2D":
                # Получаем тип аттрибута
                uniformType = words[i]

                # Получаем имя аттрибута
                i += 1
                uniformName = words[i].replace(";", "")

                # Добавляем аттрибут к тексту нового шейдера
                if (uniformName not in uniformsMap):
                    if uniformName in mainFunctionText:
                        uniformsMap[uniformName] = {"type": uniformType, "name": uniformName}
                        uniformsNamesList.append(uniformName)
                    else:
                        print("Unused uniform variable %s in shader %s" % (uniformName, inputPath))
            else:
                # Получаем имя семплера
                i += 1
                samplerName = words[i].replace(";", "")

                # Добавляем аттрибут к тексту нового шейдера
                if samplerName not in samplersNamesList:
                    samplersNamesList.append(samplerName)

        if words[i] == "varying":
            # TODO: ???
            # Если после varying идет описание точности - пропускаем его
            i += 1
            if words[i] in precisionWords:
                i += 1

            # Получаем тип аттрибута
            varyingType = words[i]

            # Получаем имя аттрибута
            i += 1
            varyingName = words[i].replace(";", "")

            # Может быть у нас массив varying переменных
            if "[" in varyingName:
                varyingMatches = re.search("([a-zA-Z_]+)\[([0-9]+)\]", varyingName)
                if varyingMatches:
                    name = varyingMatches.group(1)
                    count = int(varyingMatches.group(2))

                    if name in mainFunctionText:
                        for index in range(0, count):
                            varyingCountName = "%s_%d" % (name, index)

                            # Добавляем аттрибут к тексту нового шейдера
                            if varyingCountName not in varyingsMap:
                                if isVertexShader:
                                    resultVaryingLocations[varyingCountName] = varyingIndex
                                    newShaderVariableName = "layout(location = %d) out %s %s;\n" % (varyingIndex, varyingType, varyingCountName)
                                else:
                                    if varyingCountName in inputVaryingLocations:
                                        inputIndex = inputVaryingLocations[varyingCountName]
                                        newShaderVariableName = "layout(location = %d) in %s %s;\n" % (inputIndex, varyingType, varyingCountName)
                                    else:
                                        print("Is not compatible varyings for %s with vertex shader" % inputPath)
                                        sys.exit(2)

                                varyingsMap[varyingCountName] = newShaderVariableName
                                varyingsNamesList.append(varyingCountName)
                                varyingIndex += 1

                                # Замена в тексте
                                expression = "%s\[[ ]*%d[ ]*\]" % (name, index)
                                mainFunctionText = re.sub(expression, varyingCountName, mainFunctionText)
            else:
                # Добавляем аттрибут к тексту нового шейдера
                if (varyingName not in varyingsMap) and (varyingName in mainFunctionText):
                    newShaderVariableName = ""
                    if isVertexShader:
                        resultVaryingLocations[varyingName] = varyingIndex
                        newShaderVariableName = "layout(location = %d) out %s %s;\n" % (varyingIndex, varyingType, varyingName)
                    else:
                        if varyingName in inputVaryingLocations:
                            inputIndex = inputVaryingLocations[varyingName]
                            newShaderVariableName = "layout(location = %d) in %s %s;\n" % (inputIndex, varyingType, varyingName)
                        else:
                            print("Is not compatible varyings for %s with vertex shader" % inputPath)
                            sys.exit(2)

                    varyingsMap[varyingName] = newShaderVariableName
                    varyingsNamesList.append(varyingName)
                    varyingIndex += 1
        i += 1

    # Result shader header
    resultShaderText = "#version 450\n\n"
                        #"#extension GL_ARB_separate_shader_objects : enable\n\n"

    if isVertexShader:
        resultShaderText += "// Vertex shader\n\n"

        # Defines
        if len(definesNamesList) > 0:
            resultShaderText += "// Defines\n"
            for defineName in definesNamesList:
                resultShaderText += definesMap[defineName]
            resultShaderText += "\n"

        # Attributes
        if len(attributesNamesList) > 0:
            resultShaderText += "// Input\n"
            for attributeString in attributesNamesList:
                resultShaderText += attributesMap[attributeString]
            resultShaderText += "\n"

        # Uniforms
        if usePushConstant:
            # Сортируем по убыванию размера
            def sortFunction(uniformName):
                return convertTypeToSortPriority(uniformsMap[uniformName]["type"])

            if len(uniformsNamesList) > 0:
                uniformsNamesList = sorted(uniformsNamesList, key=sortFunction, reverse=True)

                pushConstantsText = ""

                for uniformName in uniformsNamesList:
                    uniformDict = uniformsMap[uniformName]
                    newShaderVariableName = "    layout(offset = %d) %s %s;\n" % (
                    pushConstantOffset, uniformDict["type"], uniformDict["name"])
                    pushConstantOffset += convertTypeToSize(uniformDict["type"])
                    # Only used uniforms
                    pushConstantsText += newShaderVariableName

                if len(pushConstantsText) > 0:
                    resultShaderText += "// Push constants\n" \
                                        "layout(push_constant) uniform PushConstants {\n"
                    resultShaderText += pushConstantsText
                    resultShaderText += "} uni;\n\n"  # TODO: ???
        else:
            # Сортируем по убыванию размера
            def sortFunction(uniformName):
                return convertTypeToSortPriority(fullUniformsDict[uniformName])

            uniformsNamesList = fullUniformsDict.keys()
            uniformsNamesList = sorted(uniformsNamesList, key=sortFunction, reverse=True)

            pushConstantsText = ""

            for uniformName in uniformsNamesList:
                uniformType = fullUniformsDict[uniformName]
                newShaderVariableName = "    %s %s;\n" % (uniformType, uniformName)
                pushConstantsText += newShaderVariableName

            if len(pushConstantsText) > 0:
                resultShaderText += "// Uniform buffer\n" \
                                    "layout(set = 0, binding = 0) uniform UniformBufferObject {\n"
                resultShaderText += pushConstantsText
                resultShaderText += "} uni;\n\n"  # TODO: ???

        # Varying
        if len(varyingsNamesList) > 0:
            resultShaderText += "// Varying variables\n"
            for varyingName in varyingsNamesList:
                resultShaderText += varyingsMap[varyingName]
            resultShaderText += "\n"

        # Выходные переменные
        resultShaderText += "// Vertex output\n" \
                            "out gl_PerVertex {\n" \
                            "    vec4 gl_Position;\n" \
                            "};\n"
    else:
        resultShaderText += "// Fragment shader\n\n"

        # Defines
        if len(definesNamesList) > 0:
            resultShaderText += "// Defines\n"
            for defineName in definesNamesList:
                resultShaderText += definesMap[defineName]
            resultShaderText += "\n"

        # Varying
        if len(varyingsNamesList) > 0:
            resultShaderText += "// Varying variables\n"
            for varyingName in varyingsNamesList:
                resultShaderText += varyingsMap[varyingName]
            resultShaderText += "\n"

        # Uniforms
        if usePushConstant:
            # Сортируем по убыванию размера
            def sortFunction(uniformName):
                return convertTypeToSortPriority(uniformsMap[uniformName]["type"])

            if len(uniformsNamesList) > 0:
                uniformsNamesList = sorted(uniformsNamesList, key=sortFunction, reverse=True)

                pushConstantsText = ""

                for uniformName in uniformsNamesList:
                    uniformDict = uniformsMap[uniformName]
                    newShaderVariableName = "    layout(offset = %d) %s %s;\n" % (pushConstantOffset, uniformDict["type"], uniformDict["name"])
                    pushConstantOffset += convertTypeToSize(uniformDict["type"])
                    # Only used uniforms
                    pushConstantsText += newShaderVariableName

                if len(pushConstantsText) > 0:
                    resultShaderText += "// Push constants\n" \
                                        "layout(push_constant) uniform PushConstants {\n"
                    resultShaderText += pushConstantsText
                    resultShaderText += "} uni;\n\n"  # TODO: ???
        else:
            # Сортируем по убыванию размера
            def sortFunction(uniformName):
                return convertTypeToSortPriority(fullUniformsDict[uniformName])

            uniformsNamesList = fullUniformsDict.keys()
            uniformsNamesList = sorted(uniformsNamesList, key=sortFunction, reverse=True)

            pushConstantsText = ""

            for uniformName in uniformsNamesList:
                uniformType = fullUniformsDict[uniformName]
                newShaderVariableName = "    %s %s;\n" % (uniformType, uniformName)
                pushConstantsText += newShaderVariableName

            if len(pushConstantsText) > 0:
                resultShaderText += "// Uniform buffer\n" \
                                    "layout(set = 0, binding = 0) uniform UniformBufferObject {\n"
                resultShaderText += pushConstantsText
                resultShaderText += "} uni;\n\n"  # TODO: ???
                setIndex += 1

        # Samplers
        if len(samplersNamesList) > 0:
            resultShaderText += "// Samplers\n"
            for samplerName in samplersNamesList:
                resultShaderText += "layout(set = %s, binding = 0) uniform sampler2D %s;\n" % (setIndex, samplerName)
                setIndex += 1
            resultShaderText += "\n"

        # Выходные переменные
        resultShaderText += "// Fragment output\n" \
                            "layout(location = 0) out vec4 outputFragColor;\n"


    functionDeclaration = RE5.search(mainFunctionText).group(0)

    # Function declaration replace
    mainFunctionText = mainFunctionText.replace(functionDeclaration, "void main(void) {")

    # Replace uniforms on push constants
    for uniformName in uniformsNamesList:
        #expression = "[^a-zA-Z_](%s)[^a-zA-Z_]" % uniformName
        expression = r"[\+\-\ * \ /(<>=](%s)[\+\-\ * \ /, ;.\[)<>=]" % uniformName
        replaceValue = "uni.%s" % uniformName

        matches = re.search(expression, mainFunctionText)

        while matches:
            for groupNum in range(0, len(matches.groups())):
                groupNum = groupNum + 1
                start = matches.start(groupNum)
                end = matches.end(groupNum)
                # group = matches.group(groupNum)
                mainFunctionText = mainFunctionText[0:start] + replaceValue + mainFunctionText[end:]
            matches = re.search(expression, mainFunctionText)

        # mainFunctionText = re.sub(expression, mainFunctionText, replaceValue)
        # mainFunctionText = mainFunctionText.replace(uniformName, "pc."+uniformName)

    # Fragment out variable
    if isVertexShader == False:
        mainFunctionText = mainFunctionText.replace("texture2D", "texture")
        mainFunctionText = mainFunctionText.replace("gl_FragColor", "outputFragColor")

    resultShaderText += "\n// Main function\n"
    resultShaderText += mainFunctionText

    # Сохраняем
    with open(outputPath, "w") as file:
        file.write(resultShaderText)

    # Выравнивание
    if (pushConstantOffset % 16) != 0:
        pushConstantOffset += 16
        pushConstantOffset -= pushConstantOffset % 16

    return setIndex, pushConstantOffset, resultVaryingLocations


def processShadersFolder(inputPath, outputPath):
    args = []
    for root, dirs, files in os.walk(inputPath, topdown=True):
        for fileName in files:
            if (not fileName.startswith(".")) and (fileName.endswith(".psh")):
                args.append((root, fileName, inputPath, outputPath))
                
    def threadFunc(root, fileName, inputPath, outputPath):
        resultFolder = root.replace(inputPath, outputPath)

        sourceFragmentFilePath = os.path.join(root, fileName)
        sourceVertexFilePath = sourceFragmentFilePath.replace(".psh", ".vsh")
        resultFragmentFilePath = os.path.join(resultFolder, fileName).replace(".psh", ".frag")
        resultVertexFilePath = os.path.join(resultFolder, fileName).replace(".psh", ".vert")

        # Проверка налиция обоих файлов
        if not os.path.exists(sourceVertexFilePath) or not os.path.exists(sourceFragmentFilePath):
            print("Missing shaders %s + %s" % (sourceVertexFilePath, sourceFragmentFilePath))
            sys.exit(2)

        vertexUniformsSize, vertexUniforms = analyseUniformsSize(sourceVertexFilePath)
        fragmentUniformsSize, fragmentUniforms = analyseUniformsSize(sourceFragmentFilePath)
        totalUniformsSize = vertexUniformsSize + fragmentUniformsSize

        fullUniformsDict = {}

        usePushConstants = False
        if totalUniformsSize <= 128:
            usePushConstants = True
        else:
            fullUniformsDict.update(vertexUniforms)
            fullUniformsDict.update(fragmentUniforms)

        # Обработка шейдеров
        setIndex, pushConstantOffset, varyingLocations = processShaderFile(True, sourceVertexFilePath, resultVertexFilePath, 0, [], fullUniformsDict, 0, usePushConstants)
        processShaderFile(False, sourceFragmentFilePath, resultFragmentFilePath, setIndex, varyingLocations, fullUniformsDict, pushConstantOffset, usePushConstants)


    MAX_THREADS = multiprocessing.cpu_count()
    with concurrent.futures.ThreadPoolExecutor(max_workers=MAX_THREADS) as executor:
        for params in args:
            executor.submit(threadFunc, *params)


def convertToSpirvShaders(inputFolder, outputFolder):
    scriptPath = os.path.dirname(os.path.realpath(__file__))

    converterUtilPath = ""
    if platform.system() == "Linux":
        converterUtilPath = os.path.join(scriptPath, "LinuxConverter/glslangValidator")
    elif platform.system() == "Darwin":
        converterUtilPath = os.path.join(scriptPath, "OSXConverter/glslangValidator")

    args = []
    for root, dirs, files in os.walk(inputFolder, topdown=True):
        for fileName in files:
            if (not fileName.startswith(".")) and ((".frag" in fileName) or (".vert" in fileName)):

                args.append((root, fileName, inputFolder, outputFolder))

    def threadFunc(root, fileName, inputFolder, outputFolder):
        sourceFilePath = os.path.join(root, fileName)

        resultFilePath = os.path.join(outputFolder, fileName)
        resultFilePath = resultFilePath.replace(".vert", "_v.spv")
        resultFilePath = resultFilePath.replace(".frag", "_p.spv")

        # os.system(converterUtilPath + " -V " + sourceFilePath + " -o " + resultFilePath)
        # FNULL = open(os.devnull, 'w')
        # subprocess.call([converterUtilPath, "-V", sourceFilePath, "-o", resultFilePath], stdout=FNULL, stderr=subprocess.STDOUT)
        # p = subprocess.call([converterUtilPath, "-V", sourceFilePath, "-o", resultFilePath])
        p = subprocess.Popen([converterUtilPath, "-V", sourceFilePath, "-o", resultFilePath], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        output, err = p.communicate(b"Input data that is passed to subprocess' stdin")
        if output != (bytes(sourceFilePath, 'utf-8') + b'\n'):
            print("----- Shader compilation ERRROR! -----")
            print(output.decode("utf-8"), file=sys.stderr)
            print("--------------------------------------")

        #print("%s converted" % fileName)

    MAX_THREADS = multiprocessing.cpu_count()
    with concurrent.futures.ThreadPoolExecutor(max_workers=MAX_THREADS) as executor:
        for params in args:
            executor.submit(threadFunc, *params)


if __name__ == '__main__':
    # Params
    exampleString = "main.py -i <input files folder> -o <output files folder>"
    try:
        opts, args = getopt.getopt(sys.argv[1:], "i:o:", ["ifolder=", "ofolder="])
    except getopt.GetoptError:
        print(exampleString)
        sys.exit(2)

    # Check parameters length
    if len(opts) < 2:
        print(exampleString)
        sys.exit(2)

    # Parse parameters
    inputFolder = ''
    outputFolder = ''
    for opt, arg in opts:
        if opt in ("-i", "--ifolder"):
            inputFolder = arg
        elif opt in ("-o", "--ofolder"):
            outputFolder = arg

    if not os.path.exists(outputFolder):
        os.makedirs(outputFolder)

    # Process config
    if inputFolder and outputFolder:
        tempFolderPath = os.path.join(tempfile.gettempdir(), "vulkanShaders")
        # tempFolderPath = "/tmp/vulkanShaders"

        shutil.rmtree(tempFolderPath, ignore_errors=True)
        os.mkdir(tempFolderPath)

        # Resources processing
        processShadersFolder(inputFolder, tempFolderPath)

        # Convert to Spirv
        convertToSpirvShaders(tempFolderPath, outputFolder)

        shutil.rmtree(tempFolderPath, ignore_errors=True)
