CXX := g++
CXXFLAGS += -std=c++11
CXXFLAGS += -s -O2 -shared -fPIC

SRCS := $(wildcard *.cpp)
DYLIBS := $(SRCS:%.cpp=%.so)

all: $(DYLIBS)

$(DYLIBS): %.so: %.cpp
	$(CXX) $(CXXFLAGS) $< -o $@


.PHONY: clean
clean:
	rm $(DYLIBS)
